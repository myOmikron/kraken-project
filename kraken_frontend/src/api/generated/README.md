# How to generate / update this directory?

## JetBrains

1. Add a new run configuration `Swagger Codegen`
2. Point the `Specification Path` to your openapi json downloaded from swagger
3. Point the `Generate Files to` to this directory (i.e. `<repo>/kraken_frontend/src/api/generated`)
4. Set the `Language` to `typescript-fetch`
5. Configure the generator with `.json` file using this directory's `config.json` (i.e. `<repo>/kraken_frontend/src/api/generated/config.json`)
6. Happy Generating

## CLI

[Installation instructions](https://openapi-generator.tech/docs/installation)

```bash
openapi-generator-cli generate -g typescript-fetch -i <openapi.json> -o <generated/> -c <config.json>
```