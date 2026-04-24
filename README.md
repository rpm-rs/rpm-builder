# rpm-builder

Build rpms without SPEC files. This is written in pure Rust, so it should be
easy to do static linking.

Available on COPR: https://copr.fedorainfracloud.org/coprs/dalley/rpm-builder/

## Example

```bash
rpm-builder \
  --exec-file "path/to/binary:/usr/bin/awesome-bin" \
  --config-file "path/to/config-file:/etc/awesome/config.json" \
  --doc-file "path/to/doc-file:/usr/share/man/awesome/manpage.1.gz" \
  --compression gzip \
  --changelog "me:was awesome, eh?:2018-01-02" \
  --changelog "you:yeah:2018-01-02" \
  --requires "wget >= 1.0.0" \
  --obsoletes "rpmbuild" \
  awesome
# creates a file called awesome-1.0.0-1.noarch.rpm with version 1.0.0, release 1, license is MIT.
```

## Additional Flags

| Flag                | Description                                                                                                      |
| ---                 | ---                                                                                                              |
| `arch`              | Specify the target architecture                                                                                  |
| `changelog`         | Add a changelog entry to the rpm. The entry has the form `<author>:<content>:<yyyy-mm-dd>` (time is in utc)      |
| `compression`       | Specify the compression algorithm. Currently only gzip, zstd, and "none" are supported                           |
| `config-file`       | Add a config-file to the rpm                                                                                     |
| `conflicts`         | Indicates that the rpm conflicts with another package. Use the format `<name> [> \| >= \| = \| <= \| < version]` |
| `summary`           | Give a basic description of the package (will also be used for package "description")                            |
| `dir`               | Add a directory and all its files to the rpm. Use the format `<source_dir_path>:<target_dir_path>`               |
| `doc-dir`           | Add a directory of documentation files to the rpm. Use the format `<source_dir_path>:<target_dir_path>`          |
| `enhances`          | Indicates that the rpm enhances another package. Use the format `<name> [> \| >= \| = \| <= \| < version]`       |
| `config-dir`        | Add a directory of config files to the rpm. Use the format `<source_dir_path>:<target_dir_path>`                 |
| `doc-file`          | Add a documentation-file to the rpm. Use the format `<source_path>:<target_location>`                            |
| `exec-file`         | Add a executable-file to the rpm. Use the format `<source_path>:<target_location>`                               |
| `file`              | Add a regular file to the rpm. Use the format `<source_path>:<target_location>`                                  |
| `license`           | Specify a license                                                                                                |
| `name`              | Specify the name of your package                                                                                 |
| `obsoletes`         | Indicates that the rpm obsoletes another package. Use the format `<name> [> \| >= \| = \| <= \| < version]`      |
| `out`               | Specify an out file                                                                                              |
| `provides`          | Indicates that the rpm provides another package. Use the format `<name> [> \| >= \| = \| <= \| < version]`       |
| `release`           | Specify release number of the package                                                                            |
| `recommends`        | Indicates that the rpm recommends another package. Use the format `<name> [> \| >= \| = \| <= \| < version]`     |
| `requires`          | Indicates that the rpm requires another package. Use the format `<name> [> \| >= \| = \| <= \| < version]`       |
| `rpm-format`        | Specify which version of the RPM package specification to use when building the package                          |
| `sign-with-pgp-asc` | Sign package with the specified pgp key                                                                          |
| `suggests`          | Indicates that the rpm suggests another package. Use the format `<name> [> \| >= \| = \| <= \| < version]`       |
| `supplements`       | Indicates that the rpm supplements another package. Use the format `<name> [> \| >= \| = \| <= \| < version]`    |
| `version`           | Specify a version                                                                                                |
