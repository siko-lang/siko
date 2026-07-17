![Logo](logo.png)

# Siko programming language

Status: ![](https://github.com/siko-lang/siko/workflows/Master/badge.svg)

## Testing

Run the full test suite:

```sh
make test
```

`make test` builds `siko.bin` and `runner.bin`, runs the standard library's own
`std/Common` tests, then runs the snapshot suite.

For targeted snapshot runs, build the runner and pass substring filters:

```sh
make runner.bin
./runner.bin echo5
./runner.bin --c ./siko2.bin typecheck # runs everything containing typecheck using siko2.bin as compiler
```

## License

MIT

## Community

[Discord](https://discord.com/invite/Gfd8YDrYVC)

## Website

https://www.siko-lang.org
