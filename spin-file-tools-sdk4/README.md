# spin-file-tools

Minimalna usługa Spin/Rust pod `spin-sdk = 4.0.0`.

Funkcje:
- walidacja JSON,
- prosta walidacja dokumentu prostym schematem,
- konwersja JSON -> CSV,
- konwersja CSV -> JSON,
- metadata obrazu,
- grayscale obrazu,
- resize obrazu.

## Build

```powershell
rustup target add wasm32-wasip2
spin build
spin up
```

Albo:

```powershell
cargo build --target wasm32-wasip2 --release
spin up
```

## Endpointy

```text
GET  /health
GET  /version
GET  /routes
POST /validate/json
POST /convert/json-to-csv
POST /convert/csv-to-json
POST /image/metadata
POST /image/grayscale?format=png|jpeg
POST /image/resize?width=256&height=256&format=png|jpeg
```

## Testy

```powershell
curl.exe http://localhost:3000/health
curl.exe http://localhost:3000/routes
```

JSON validation:

```powershell
curl.exe `
  -X POST "http://localhost:3000/validate/json" `
  -H "Content-Type: application/json" `
  --data-binary '{ "id": 1, "title": "The Matrix" }'
```

Simple schema validation:

```powershell
curl.exe `
  -X POST "http://localhost:3000/validate/json" `
  -H "Content-Type: application/json" `
  --data-binary '{ "schema": { "type": "object", "required": ["id", "title"], "properties": { "id": { "type": "integer" }, "title": { "type": "string" } } }, "document": { "id": 1, "title": "The Matrix" } }'
```

JSON -> CSV:

```powershell
@'
[
  { "id": 1, "title": "The Matrix", "year": 1999 },
  { "id": 2, "title": "Alien", "year": 1979 }
]
'@ | Set-Content .\sample.json -Encoding UTF8

curl.exe `
  -X POST "http://localhost:3000/convert/json-to-csv" `
  -H "Content-Type: application/json" `
  --data-binary "@sample.json"
```

CSV -> JSON:

```powershell
@'
id,title,year
1,The Matrix,1999
2,Alien,1979
'@ | Set-Content .\sample.csv -Encoding UTF8

curl.exe `
  -X POST "http://localhost:3000/convert/csv-to-json" `
  -H "Content-Type: text/csv" `
  --data-binary "@sample.csv"
```

Image metadata:

```powershell
curl.exe `
  -X POST "http://localhost:3000/image/metadata" `
  -H "Content-Type: image/png" `
  --data-binary "@input.png"
```

Grayscale:

```powershell
curl.exe `
  -X POST "http://localhost:3000/image/grayscale?format=png" `
  -H "Content-Type: image/png" `
  --data-binary "@input.png" `
  --output grayscale.png
```

Resize:

```powershell
curl.exe `
  -X POST "http://localhost:3000/image/resize?width=256&height=256&format=png" `
  -H "Content-Type: image/png" `
  --data-binary "@input.png" `
  --output resized.png
```
