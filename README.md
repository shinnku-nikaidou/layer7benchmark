# Layer7Benchmark

## Usage

```bash
./layer7benchmark -c [concurrent-count] -a [url] -t [time]

-c or --concurrent-count u8
    concurrent thread count for download (default is 2)
-u or --url string
    url to download (default is https://www.google.com)
-t or --time u32
    time to download (default is 60)
-h or --help
    help for layer7benchmark
-v or --version
    version for layer7benchmark
--header string
    http header to send
-i or --ip string
    ip to send the request to (default is automatically resolved from the url)
```
