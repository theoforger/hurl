/*
 * hurl (https://hurl.dev)
 * Copyright (C) 2020 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
extern crate libxml;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
#[allow(unused)]
use std::io::prelude::*;
use std::io::Read;
use std::path::Path;

use crate::ast::*;
use crate::http;

use super::core::{Error, RunnerError};
use super::value::Value;

impl MultipartParam {
    pub fn eval(
        self,
        variables: &HashMap<String, Value>,
        context_dir: String,
    ) -> Result<http::MultipartParam, Error> {
        match self {
            MultipartParam::Param(KeyValue { key, value, .. }) => {
                let name = key.value;
                let value = value.eval(variables)?;
                Ok(http::MultipartParam::Param(http::Param { name, value }))
            }
            MultipartParam::FileParam(param) => {
                let file_param = param.eval(context_dir)?;
                Ok(http::MultipartParam::FileParam(file_param))
            }
        }
    }
}

impl FileParam {
    pub fn eval(self, context_dir: String) -> Result<http::FileParam, Error> {
        let name = self.key.value;

        let filename = self.value.filename.clone();
        let path = Path::new(filename.value.as_str());
        let absolute_filename = if path.is_absolute() {
            filename.value.clone()
        } else {
            Path::new(context_dir.as_str())
                .join(filename.value.clone())
                .to_str()
                .unwrap()
                .to_string()
        };

        let data = match File::open(absolute_filename.clone()) {
            Ok(mut f) => {
                let mut bytes = Vec::new();
                match f.read_to_end(&mut bytes) {
                    Ok(_) => bytes,
                    Err(_) => {
                        return Err(Error {
                            source_info: filename.source_info,
                            inner: RunnerError::FileReadAccess {
                                value: absolute_filename,
                            },
                            assert: false,
                        })
                    }
                }
            }
            Err(_) => {
                return Err(Error {
                    source_info: filename.source_info,
                    inner: RunnerError::FileReadAccess {
                        value: absolute_filename,
                    },
                    assert: false,
                })
            }
        };

        if !Path::new(&absolute_filename).exists() {
            return Err(Error {
                source_info: filename.source_info,
                inner: RunnerError::FileReadAccess {
                    value: filename.value.clone(),
                },
                assert: false,
            });
        }

        let content_type = self.value.content_type();
        Ok(http::FileParam {
            name,
            filename: filename.value,
            data,
            content_type,
        })
    }
}

impl FileValue {
    pub fn content_type(&self) -> String {
        match self.content_type.clone() {
            None => match Path::new(self.filename.value.as_str())
                .extension()
                .and_then(OsStr::to_str)
            {
                Some("gif") => "image/gif".to_string(),
                Some("jpg") => "image/jpeg".to_string(),
                Some("jpeg") => "image/jpeg".to_string(),
                Some("png") => "image/png".to_string(),
                Some("svg") => "image/svg+xml".to_string(),
                Some("txt") => "text/plain".to_string(),
                Some("htm") => "text/html".to_string(),
                Some("html") => "text/html".to_string(),
                Some("pdf") => "application/pdf".to_string(),
                Some("xml") => "application/xml".to_string(),
                _ => "application/octet-stream".to_string(),
            },
            Some(content_type) => content_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::SourceInfo;

    use super::*;

    pub fn whitespace() -> Whitespace {
        Whitespace {
            value: String::from(" "),
            source_info: SourceInfo::init(0, 0, 0, 0),
        }
    }

    #[test]
    pub fn test_eval_file_param() {
        let line_terminator = LineTerminator {
            space0: whitespace(),
            comment: None,
            newline: whitespace(),
        };
        assert_eq!(
            FileParam {
                line_terminators: vec![],
                space0: whitespace(),
                key: EncodedString {
                    value: "upload1".to_string(),
                    encoded: "upload1".to_string(),
                    quotes: false,
                    source_info: SourceInfo::init(0, 0, 0, 0),
                },
                space1: whitespace(),
                space2: whitespace(),
                value: FileValue {
                    space0: whitespace(),
                    filename: Filename {
                        value: "hello.txt".to_string(),
                        source_info: SourceInfo::init(0, 0, 0, 0)
                    },
                    space1: whitespace(),
                    space2: whitespace(),
                    content_type: None,
                },
                line_terminator0: line_terminator,
            }
            .eval("integration/tests".to_string())
            .unwrap(),
            http::FileParam {
                name: "upload1".to_string(),
                filename: "hello.txt".to_string(),
                data: b"Hello World!".to_vec(),
                content_type: "text/plain".to_string(),
            }
        );
    }

    #[test]
    pub fn test_file_value_content_type() {
        assert_eq!(
            FileValue {
                space0: whitespace(),
                filename: Filename {
                    value: "hello.txt".to_string(),
                    source_info: SourceInfo::init(0, 0, 0, 0),
                },
                space1: whitespace(),
                space2: whitespace(),
                content_type: None,
            }
            .content_type(),
            "text/plain".to_string()
        );

        assert_eq!(
            FileValue {
                space0: whitespace(),
                filename: Filename {
                    value: "hello.html".to_string(),
                    source_info: SourceInfo::init(0, 0, 0, 0),
                },
                space1: whitespace(),
                space2: whitespace(),
                content_type: None,
            }
            .content_type(),
            "text/html".to_string()
        );

        assert_eq!(
            FileValue {
                space0: whitespace(),
                filename: Filename {
                    value: "hello.txt".to_string(),
                    source_info: SourceInfo::init(0, 0, 0, 0),
                },
                space1: whitespace(),
                space2: whitespace(),
                content_type: Some("text/html".to_string()),
            }
            .content_type(),
            "text/html".to_string()
        );

        assert_eq!(
            FileValue {
                space0: whitespace(),
                filename: Filename {
                    value: "hello".to_string(),
                    source_info: SourceInfo::init(0, 0, 0, 0),
                },
                space1: whitespace(),
                space2: whitespace(),
                content_type: None,
            }
            .content_type(),
            "application/octet-stream".to_string()
        );
    }
}
