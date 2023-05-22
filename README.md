# ResourcepackD (rpd)

An extremely simple CLI utility to watch and compile minecraft data and resource packs on the fly!

RPD supports optional validation of JSON files and stripping comments from JSONC files, however other contents are not checked and are untouched. If you wish to use your pack in production I recommend to further use [PackSquash](https://github.com/ComunidadAylas/PackSquash) on it.

## Usage

Install using cargo or just [download release binaries](https://github.com/Maxuss/resourcepackd/releases/latest) on Github

```sh
cargo install resourcepackd
```

You can then watch your directory

```sh
rpd watch <ROOT_DIR> -o <OUT_FILE>
```

e.g.

```sh
rpd watch . -o build/resourcepack.zip
```

RPD will launch and you will be able to start developing your resourcepack!

You can also do one-time compilation with

```sh
rpd compile . -o build/resourcepack.zip
```

## Enabling Validation

To validate JSON files and clean JSONC files you can add the `-v` flag:

```sh
rpd watch . -o build/resourcepack.zip -v
```

This will automatically perform validation on all `.json` and `.mcmeta` files, as well as JSONC comment stripping on all `.jsonc` and `.mcmetac` files.

