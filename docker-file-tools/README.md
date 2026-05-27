# docker-file-tools

Wersja native/Docker dla usługi `spin-file-tools`.

Cel: porównanie z wersją Spin/Wasm przy możliwie tej samej logice:
- walidacja JSON,
- JSON -> CSV,
- CSV -> JSON,
- metadata obrazu,
- grayscale,
- resize.

Port domyślny: `8081`.

## Uruchomienie lokalnie bez Dockera

```powershell
cargo run --release
```

Test:

```powershell
curl.exe http://localhost:8081/health
```

## Build i uruchomienie Dockera

```powershell
docker build -t docker-file-tools:latest .
docker run --rm -p 8081:8081 docker-file-tools:latest
```

## Endpointy

```text
GET  /health
GET  /version
GET  /routes
GET  /ping
POST /echo
POST /validate/json
POST /convert/json-to-csv
POST /convert/csv-to-json
POST /image/metadata
POST /image/grayscale?format=png|jpeg
POST /image/resize?width=256&height=256&format=png|jpeg
```

## Testy PowerShell / curl.exe

Health:

```powershell
curl.exe http://localhost:8081/health
```

Routes:

```powershell
curl.exe http://localhost:8081/routes
```

JSON validation:

```powershell
curl.exe `
  -X POST "http://localhost:8081/validate/json" `
  -H "Content-Type: application/json" `
  --data-binary '{ "id": 1, "title": "The Matrix" }'
```

Simple schema validation:

```powershell
curl.exe `
  -X POST "http://localhost:8081/validate/json" `
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
  -X POST "http://localhost:8081/convert/json-to-csv" `
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
  -X POST "http://localhost:8081/convert/csv-to-json" `
  -H "Content-Type: text/csv" `
  --data-binary "@sample.csv"
```

Image metadata:

```powershell
curl.exe `
  -X POST "http://localhost:8081/image/metadata" `
  -H "Content-Type: image/png" `
  --data-binary "@input.png"
```

Grayscale:

```powershell
curl.exe `
  -X POST "http://localhost:8081/image/grayscale?format=png" `
  -H "Content-Type: image/png" `
  --data-binary "@input.png" `
  --output grayscale-docker.png
```

Resize:

```powershell
curl.exe `
  -X POST "http://localhost:8081/image/resize?width=256&height=256&format=png" `
  -H "Content-Type: image/png" `
  --data-binary "@input.png" `
  --output resized-docker.png
```

## Benchmark prosto z PowerShell

Spin:

```powershell
Measure-Command {
  curl.exe `
    -X POST "http://localhost:3000/image/resize?width=256&height=256&format=png" `
    -H "Content-Type: image/png" `
    --data-binary "@input.png" `
    --output NUL
}
```

Docker:

```powershell
Measure-Command {
  curl.exe `
    -X POST "http://localhost:8081/image/resize?width=256&height=256&format=png" `
    -H "Content-Type: image/png" `
    --data-binary "@input.png" `
    --output NUL
}
```

Odpowiedzi transformacji zawierają nagłówek:

```text
x-internal-processing-us
```

To przybliżony czas samej logiki po stronie aplikacji, bez czasu klienta i części narzutu HTTP.
