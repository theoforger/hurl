GET http://localhost:8000/redirect-3
[Options]
location: true
HTTP 200
[Asserts]
redirects count == 3
redirects nth 0 url == "http://localhost:8000/redirect-3"
redirects nth 1 url == "http://localhost:8000/redirect-2"
redirects nth 2 url == "http://localhost:8000/redirect-1"
redirects nth 0 url contains "redirect-3"
redirects nth 1 url endsWith "2"
redirects nth 2 url matches /^http:\/\/.*redirect-1$/
url == "http://localhost:8000/redirected"
`Redirected`
