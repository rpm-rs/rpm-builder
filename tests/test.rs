use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::{Command, cargo_bin};
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use tempdir::TempDir;

/// Test default behavior w/ minimum possible input provided, e.g. version, release auto-fill
#[test]
fn test_basic_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("rpm-builder-test-basic-defaults")?;
    let out_file = tmp_dir.path().join("test-1.0.0-1.noarch.rpm");

    assert!(!fs::exists(&out_file).unwrap());
    Command::new(cargo_bin!())
        .arg("test")
        .arg("-o")
        .arg(&out_file)
        .assert()
        .success();
    assert!(fs::exists(&out_file).unwrap());

    let pkg = rpm::Package::open(&out_file).expect("couldn't find package");
    assert_eq!(pkg.metadata.get_name()?, "test");
    assert_eq!(pkg.metadata.get_epoch()?, 0);
    assert_eq!(pkg.metadata.get_version()?, "1.0.0");
    assert_eq!(pkg.metadata.get_release()?, "1");
    assert_eq!(pkg.metadata.get_arch()?, "noarch");
    assert_eq!(pkg.metadata.get_license()?, "MIT"); // the default (todo: maybe shouldn't have a default?)
    assert_eq!(pkg.metadata.get_summary()?, "");
    assert_eq!(pkg.metadata.get_description()?, ""); // should be a copy of the summary
    assert_eq!(
        pkg.metadata.get_payload_compressor()?,
        rpm::CompressionType::Zstd
    );

    // provides itself by default
    assert_eq!(
        pkg.metadata.get_provides()?,
        vec![rpm::Dependency::eq("test", "0:1.0.0-1"),]
    );
    // has no requires by default, except for rpmlib() ones
    assert_eq!(
        pkg.metadata
            .get_requires()?
            .into_iter()
            .filter(|r| !r.flags.contains(rpm::DependencyFlags::RPMLIB))
            .collect::<Vec<rpm::Dependency>>(),
        vec![]
    );
    // no other deps by default
    assert_eq!(pkg.metadata.get_obsoletes()?, vec![]);
    assert_eq!(pkg.metadata.get_conflicts()?, vec![]);
    assert_eq!(pkg.metadata.get_suggests()?, vec![]);
    assert_eq!(pkg.metadata.get_recommends()?, vec![]);
    assert_eq!(pkg.metadata.get_supplements()?, vec![]);
    assert_eq!(pkg.metadata.get_enhances()?, vec![]);
    // no filelists or changelog entries by default
    assert_eq!(pkg.metadata.get_file_entries()?, vec![]);
    assert_eq!(pkg.metadata.get_changelog_entries()?, vec![]);

    Ok(())
}

/// Assert that the command fails if no package name was provided
#[test]
fn test_no_name_provided() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("rpm-builder-no-name-provided")?;

    Command::new(cargo_bin!())
        .arg("-o")
        .arg(&tmp_dir.path())
        .assert()
        .failure();

    Ok(())
}

/// Test adding basic metadata (version, epoch, release, arch, license, summary) to the package
#[test]
fn test_set_basic_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("rpm-builder-test-set-metadata")?;
    let out_file = tmp_dir
        .path()
        .join("test-set-metadata-2.3.4-5.fc46.x86_64.rpm");

    assert!(!fs::exists(&out_file).unwrap());
    Command::new(cargo_bin!())
        .arg("test-set-metadata")
        .arg("--epoch")
        .arg("1")
        .arg("--version")
        .arg("2.3.4")
        .arg("--release")
        .arg("5.fc46")
        .arg("--arch")
        .arg("x86_64")
        .arg("--license")
        .arg("MPL-2.0")
        .arg("--summary")
        .arg("blah blah blah")
        .arg("-o")
        .arg(&out_file)
        .assert()
        .success();
    assert!(fs::exists(&out_file).unwrap());

    let pkg = rpm::Package::open(&out_file)?;
    assert_eq!(pkg.metadata.get_name()?, "test-set-metadata");
    assert_eq!(pkg.metadata.get_epoch()?, 1);
    assert_eq!(pkg.metadata.get_version()?, "2.3.4");
    assert_eq!(pkg.metadata.get_release()?, "5.fc46");
    assert_eq!(pkg.metadata.get_arch()?, "x86_64");
    assert_eq!(pkg.metadata.get_license()?, "MPL-2.0");
    assert_eq!(pkg.metadata.get_summary()?, "blah blah blah");
    assert_eq!(pkg.metadata.get_description()?, "blah blah blah"); // should be a copy of the summary

    Ok(())
}

/// Test that the output option behaves as intended in various circumstances
#[test]
fn test_output_option() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("rpm-builder-test-outputs")?;

    // test using an explicit filename as output
    let explicit_filename = &tmp_dir.path().join("explicit-filename.rpm");
    assert!(!fs::exists(&explicit_filename).unwrap());
    Command::new(cargo_bin!())
        .arg("test-output")
        .arg("-o")
        .arg(&explicit_filename)
        .assert()
        .success();
    assert!(fs::exists(&explicit_filename).unwrap());

    let filename = Path::new("test-output-1.0.0-1.noarch.rpm");
    let file_in_tmp = &tmp_dir.path().join(&filename);

    // test using a directory as output, no provided filename
    assert!(!fs::exists(&file_in_tmp).unwrap());
    Command::new(cargo_bin!())
        .arg("test-output")
        .arg("-o")
        .arg(&tmp_dir.path())
        .assert()
        .success();
    assert!(fs::exists(&file_in_tmp).unwrap());

    // test no output option provided at all - no provided filename, current working directory
    let orig_cwd = env::current_dir()?;
    env::set_current_dir(&tmp_dir.path())?;

    let expected_filename = Path::new("test-no-output-1.0.0-1.noarch.rpm");
    assert!(!fs::exists(&expected_filename).unwrap());
    Command::new(cargo_bin!())
        .arg("test-no-output")
        .assert()
        .success();
    assert!(fs::exists(&expected_filename).unwrap());

    env::set_current_dir(orig_cwd)?;

    Ok(())
}

/// Test providing the compression option
#[test]
fn test_package_compression() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tmp_dir = TempDir::new("rpm-builder-test-compression-option")?;

    for compression in ["gzip", "zstd", "none"] {
        let out_file = tmp_dir.path().join(format!(
            "test-compression-{}-1.0.0-1.noarch.rpm",
            &compression
        ));
        assert!(!fs::exists(&out_file).unwrap());
        Command::new(cargo_bin!())
            .arg(&format!("test-compression-{}", &compression))
            .arg("--exec-file")
            .arg(&format!(
                "{}/tests/assets/multiplication_tables.py:/usr/bin/multiplication_tables",
                workspace_path.to_string_lossy()
            ))
            .arg("--compression")
            .arg(&compression)
            .arg("-o")
            .arg(&out_file)
            .assert()
            .success();
        assert!(fs::exists(&out_file).unwrap());

        let pkg = rpm::Package::open(&out_file)?;
        assert_eq!(
            pkg.metadata.get_payload_compressor()?,
            match compression {
                "none" => rpm::CompressionType::None,
                "zstd" => rpm::CompressionType::Zstd,
                "gzip" => rpm::CompressionType::Gzip,
                _ => unreachable!(),
            }
        );
    }

    // Test an invalid value for the compression option
    Command::new(cargo_bin!())
        .arg("test-compression")
        .arg("--compression")
        .arg("invalid")
        .arg("-o")
        .arg(&tmp_dir.path())
        .assert()
        .failure();

    Ok(())
}

/// Test adding changelogs to the package
#[test]
fn test_adding_changelogs() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("rpm-builder-test-adding-changelogs")?;
    let out_file = tmp_dir.path().join("test-changelogs-1.0.0-1.noarch.rpm");

    assert!(!fs::exists(&out_file).unwrap());
    Command::new(cargo_bin!())
        .arg("test-changelogs")
        .arg("--changelog")
        .arg("Walter White <ww@breakingbad.com>:I am the danger:2018-01-02")
        .arg("--changelog")
        .arg("jpinkman@breakingbad.com:yeah, science!:2019-02-03")
        .arg("-o")
        .arg(&out_file)
        .assert()
        .success();
    assert!(fs::exists(&out_file).unwrap());

    let pkg = rpm::Package::open(&out_file)?;
    assert_eq!(
        pkg.metadata.get_changelog_entries()?,
        vec![
            rpm::ChangelogEntry {
                name: "Walter White <ww@breakingbad.com>".to_owned(),
                timestamp: 1514851200,
                description: "I am the danger".to_owned()
            },
            rpm::ChangelogEntry {
                name: "jpinkman@breakingbad.com".to_owned(),
                timestamp: 1549152000,
                description: "yeah, science!".to_owned()
            }
        ]
    );

    Ok(())
}

/// Test adding dependencies / provides / conflicts / etc. to the package
#[test]
fn test_adding_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("rpm-builder-test-adding-dependencies")?;
    let out_file = tmp_dir.path().join("test-dependencies-1.0.0-1.noarch.rpm");

    assert!(!fs::exists(&out_file).unwrap());
    Command::new(cargo_bin!())
        .args(vec![
            "test-dependencies",
            "--provides",
            "/usr/bin/rpmbuilder",
            "--requires",
            "wget >= 1.0.0",
            "--obsoletes",
            "rpmbuild",
            "--conflicts",
            "foobar123",
            "--suggests",
            "foobar456 = 2.3.4",
            "--recommends",
            "foobar678 < 2.0",
            "--enhances",
            "foobar678 > 1.0",
            "--supplements",
            "foobarbaz",
            "-o",
            &out_file.to_string_lossy(),
        ])
        .assert()
        .success();
    assert!(fs::exists(&out_file).unwrap());

    let pkg = rpm::Package::open(&out_file)?;

    assert_eq!(
        pkg.metadata.get_provides()?,
        vec![
            rpm::Dependency::any("/usr/bin/rpmbuilder"),
            rpm::Dependency::eq("test-dependencies", "0:1.0.0-1"),
        ]
    );
    // has no requires by default, except for rpmlib() ones
    assert_eq!(
        pkg.metadata
            .get_requires()?
            .into_iter()
            .filter(|r| !r.flags.contains(rpm::DependencyFlags::RPMLIB))
            .collect::<Vec<rpm::Dependency>>(),
        vec![rpm::Dependency::greater_eq("wget", "1.0.0")]
    );
    assert_eq!(
        pkg.metadata.get_obsoletes()?,
        vec![rpm::Dependency::any("rpmbuild"),]
    );
    assert_eq!(
        pkg.metadata.get_conflicts()?,
        vec![rpm::Dependency::any("foobar123"),]
    );
    assert_eq!(
        pkg.metadata.get_suggests()?,
        vec![rpm::Dependency::eq("foobar456", "2.3.4")]
    );
    assert_eq!(
        pkg.metadata.get_recommends()?,
        vec![rpm::Dependency::less("foobar678", "2.0"),]
    );
    assert_eq!(
        pkg.metadata.get_enhances()?,
        vec![rpm::Dependency::greater("foobar678", "1.0"),]
    );
    assert_eq!(
        pkg.metadata.get_supplements()?,
        vec![rpm::Dependency::any("foobarbaz"),]
    );

    Ok(())
}

/// Test adding files and directories to the RPM
#[test]
fn test_adding_files() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tmp_dir = TempDir::new("rpm-builder-test-adding-files")?;
    let out_file = tmp_dir.path().join("test-adding-files-1.0.0-1.noarch.rpm");

    assert!(!fs::exists(&out_file).unwrap());
    Command::new(cargo_bin!())
        .env("SOURCE_DATE_EPOCH", "1600000000")
        .args(vec![
            "test-adding-files",
            "--exec-file",
            &format!(
                "{}/tests/assets/multiplication_tables.py:/usr/bin/multiplication_tables",
                workspace_path.to_string_lossy()
            ),
            "--config-file",
            &format!(
                "{}/tests/assets/example_config.toml:/etc/test-adding-files/example_config.toml",
                workspace_path.to_string_lossy()
            ),
            "--doc-file",
            &format!(
                "{}/tests/assets/example_config.toml:/usr/share/man/test-adding-files/example_config.toml",
                workspace_path.to_string_lossy()
            ),
            "--file",
            &format!(
                "{}/tests/assets/multiplication_tables.py:/usr/share/test-adding-files/multiplication_tables",
                workspace_path.to_string_lossy()
            ),
            "--dir",
            &format!(
                "{}/tests/assets/module:/usr/lib/test-adding-files",
                workspace_path.to_string_lossy()
            ),
            "--config-dir",
            &format!(
                "{}/tests/assets/foo:/etc/test-adding-files",
                workspace_path.to_string_lossy()
            ),
            "--doc-dir",
            &format!(
                "{}/tests/assets/foo:/usr/share/man/test-adding-files",
                workspace_path.to_string_lossy()
            ),
            "-o",
            &out_file.to_string_lossy(),
        ])
        .assert()
        .success();
    assert!(fs::exists(&out_file).unwrap());

    let pkg = rpm::Package::open(&out_file)?;
    let entries = vec![
        rpm::FileEntry {
            path: PathBuf::from("/etc/test-adding-files"),
            mode: rpm::FileMode::dir(0o755),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 0,
            flags: rpm::FileFlags::empty(),
            digest: None,
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/etc/test-adding-files/bar"),
            mode: rpm::FileMode::dir(0o755),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 0,
            flags: rpm::FileFlags::empty(),
            digest: None,
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/etc/test-adding-files/bar/a.txt"),
            mode: rpm::FileMode::regular(0o644),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 4,
            flags: rpm::FileFlags::CONFIG,
            digest: Some(rpm::FileDigest {
                digest: "f0e4c2f76c58916ec258f246851bea091d14d4247a2fc3e18694461b1816e13b"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/etc/test-adding-files/bar/b.txt"),
            mode: rpm::FileMode::regular(0o644),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 4,
            flags: rpm::FileFlags::CONFIG,
            digest: Some(rpm::FileDigest {
                digest: "f0e4c2f76c58916ec258f246851bea091d14d4247a2fc3e18694461b1816e13b"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/etc/test-adding-files/example_config.toml"),
            mode: rpm::FileMode::regular(0o644),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 31,
            flags: rpm::FileFlags::CONFIG,
            digest: Some(rpm::FileDigest {
                digest: "53a79039d2d619dd41cd04d550d94c531ec634cda9457f25031c141d8e4820e8"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/etc/test-adding-files/z.txt"),
            mode: rpm::FileMode::regular(0o644),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 4,
            flags: rpm::FileFlags::CONFIG,
            digest: Some(rpm::FileDigest {
                digest: "f0e4c2f76c58916ec258f246851bea091d14d4247a2fc3e18694461b1816e13b"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/usr/bin/multiplication_tables"),
            mode: rpm::FileMode::regular(0o755),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 118,
            flags: rpm::FileFlags::empty(),
            digest: Some(rpm::FileDigest {
                digest: "a2919ab787acdb6f6ae85a8f18c4e983745988ac6c1cd0ec75c8971196d2953c"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/usr/lib/test-adding-files"),
            mode: rpm::FileMode::dir(0o755),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 0,
            flags: rpm::FileFlags::empty(),
            digest: None,
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/usr/lib/test-adding-files/__init__.py"),
            mode: rpm::FileMode::regular(0o644),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 0,
            flags: rpm::FileFlags::empty(),
            digest: Some(rpm::FileDigest {
                digest: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/usr/lib/test-adding-files/hello.py"),
            mode: rpm::FileMode::regular(0o644),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 53,
            flags: rpm::FileFlags::empty(),
            digest: Some(rpm::FileDigest {
                digest: "b184c98581244d04ffbe7e17af060daf515a1e79f869d5ac6fffb8276ea61ca1"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/usr/share/man/test-adding-files"),
            mode: rpm::FileMode::dir(0o755),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 0,
            flags: rpm::FileFlags::empty(),
            digest: None,
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/usr/share/man/test-adding-files/bar"),
            mode: rpm::FileMode::dir(0o755),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 0,
            flags: rpm::FileFlags::empty(),
            digest: None,
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/usr/share/man/test-adding-files/bar/a.txt"),
            mode: rpm::FileMode::regular(0o644),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 4,
            flags: rpm::FileFlags::DOC,
            digest: Some(rpm::FileDigest {
                digest: "f0e4c2f76c58916ec258f246851bea091d14d4247a2fc3e18694461b1816e13b"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/usr/share/man/test-adding-files/bar/b.txt"),
            mode: rpm::FileMode::regular(0o644),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 4,
            flags: rpm::FileFlags::DOC,
            digest: Some(rpm::FileDigest {
                digest: "f0e4c2f76c58916ec258f246851bea091d14d4247a2fc3e18694461b1816e13b"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/usr/share/man/test-adding-files/example_config.toml"),
            mode: rpm::FileMode::regular(0o644),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 31,
            flags: rpm::FileFlags::DOC,
            digest: Some(rpm::FileDigest {
                digest: "53a79039d2d619dd41cd04d550d94c531ec634cda9457f25031c141d8e4820e8"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/usr/share/man/test-adding-files/z.txt"),
            mode: rpm::FileMode::regular(0o644),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 4,
            flags: rpm::FileFlags::DOC,
            digest: Some(rpm::FileDigest {
                digest: "f0e4c2f76c58916ec258f246851bea091d14d4247a2fc3e18694461b1816e13b"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
        rpm::FileEntry {
            path: PathBuf::from("/usr/share/test-adding-files/multiplication_tables"),
            mode: rpm::FileMode::regular(0o644),
            ownership: rpm::FileOwnership {
                user: "root".to_owned(),
                group: "root".to_owned(),
            },
            modified_at: rpm::Timestamp(1600000000),
            size: 118,
            flags: rpm::FileFlags::empty(),
            digest: Some(rpm::FileDigest {
                digest: "a2919ab787acdb6f6ae85a8f18c4e983745988ac6c1cd0ec75c8971196d2953c"
                    .to_owned(),
                algo: rpm::DigestAlgorithm::Sha2_256,
            }),
            caps: None,
            linkto: "".to_owned(),
            ima_signature: None,
        },
    ];
    assert_eq!(pkg.metadata.get_file_entries()?, entries);

    Ok(())
}

/// Test using the signing options
#[test]
fn test_signature() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tmp_dir = TempDir::new("rpm-builder-test-signature")?;
    let out_file = tmp_dir.path().join("test-signature-1.0.0-1.noarch.rpm");

    let private_key_path = workspace_path.join("tests/assets/package-manager.key");
    let public_key_path = workspace_path.join("tests/assets/package-manager.key.pub");

    assert!(!fs::exists(&out_file).unwrap());
    Command::new(cargo_bin!())
        .arg("test-signature")
        .arg("--sign-with-pgp-asc")
        .arg(&private_key_path)
        .arg("-o")
        .arg(&out_file)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
    assert!(fs::exists(&out_file).unwrap());

    let pkg = rpm::Package::open(&out_file)?;
    let raw_public_key = fs::read(public_key_path)?;
    let verifier = rpm::signature::pgp::Verifier::from_asc_bytes(&raw_public_key)?;
    pkg.verify_signature(verifier)?;

    Ok(())
}

/// Test the --rpm-version flag
#[test]
fn test_rpm_format() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("rpm-builder-test-rpm-format")?;

    // Test with rpm-version 6 (should contain RPMFORMAT and PAYLOADSIZE tags, use LONG* size tags)
    let out_file_v6 = tmp_dir.path().join("test-rpm-format-6-1.0.0-1.noarch.rpm");
    Command::new(cargo_bin!())
        .arg("test-rpm-format-6")
        .arg("--rpm-format")
        .arg("v6")
        .arg("-o")
        .arg(&out_file_v6)
        .assert()
        .success();
    assert!(fs::exists(&out_file_v6).unwrap());

    let pkg_v6 = rpm::PackageMetadata::open(&out_file_v6)?;

    assert!(
        pkg_v6
            .header
            .entry_is_present(rpm::IndexTag::RPMTAG_RPMFORMAT)
    );
    assert!(
        pkg_v6
            .header
            .entry_is_present(rpm::IndexTag::RPMTAG_PAYLOADSIZE)
    );
    assert!(
        pkg_v6
            .header
            .entry_is_present(rpm::IndexTag::RPMTAG_LONGSIZE)
    );

    // Test with rpm-version 4 (should not contain RPMFORMAT or PAYLOADSIZE)
    let out_file_v4 = tmp_dir.path().join("test-rpm-format-4-1.0.0-1.noarch.rpm");
    Command::new(cargo_bin!())
        .arg("test-rpm-format-4")
        .arg("--rpm-format")
        .arg("v4")
        .arg("-o")
        .arg(&out_file_v4)
        .assert()
        .success();
    assert!(fs::exists(&out_file_v4).unwrap());

    let pkg_v4 = rpm::PackageMetadata::open(&out_file_v4)?;

    assert!(
        !pkg_v4
            .header
            .entry_is_present(rpm::IndexTag::RPMTAG_RPMFORMAT)
    );
    assert!(
        !pkg_v4
            .header
            .entry_is_present(rpm::IndexTag::RPMTAG_PAYLOADSIZE)
    );
    assert!(pkg_v4.header.entry_is_present(rpm::IndexTag::RPMTAG_SIZE));

    // Test invalid rpm-version value
    Command::new(cargo_bin!())
        .arg("test-rpm-format-invalid")
        .arg("--rpm-format")
        .arg("invalid")
        .arg("-o")
        .arg(&tmp_dir.path())
        .assert()
        .failure();

    Ok(())
}
