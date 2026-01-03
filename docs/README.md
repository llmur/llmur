# LLMUR documentation

🐣 Base documentation template from [Hextra](https://github.com/imfing/hextra)

## Local Development

Pre-requisites: [Hugo](https://gohugo.io/getting-started/installing/) and [Go](https://golang.org/doc/install)

```shell
# Start the server
hugo mod tidy
hugo server --logLevel debug --disableFastRender -p 1313
```

### Update theme

```shell
hugo mod get -u
hugo mod tidy
```

See [Update modules](https://gohugo.io/hugo-modules/use-modules/#update-modules) for more details.

