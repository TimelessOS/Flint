# Architecture of Flint

## Repository

Repositories are stored on a local machine at /var/lib/flint, and ~/.local/flint/ unless otherwise specified in its library form

### Manifest

The manifest contains repository metadata, including:

- **Name**
- **Description**
- **Updates URL** (URL to a SIGNED manifest that should be used as an update)
- **Public Key** (The public key of the **NEXT** manifest)
- **Mirror URLs**
- **Minimum Flint version required**
- **Hash type** (defaults to `blake3`)
- **Package manifests**

Each package manifest includes individual metadata and a chunklist (similar to `mtree`), specifying expected permissions, a hash, and expected size in kilobytes.

### Chunks

Chunks are the basis of Flints content-addressable storage (CAS) and deduplication. Chunk filenames are derived from a hash of their contents.
Each chunk contains the raw data from the file tree.

### Summary

The on-disk structure is:

```
chunks/
manifest.yml.sig
manifest.yml
```

## Bundles

### Headers

A bundle "header" is a prefixed bundle extractor and runner, allowing quick execution without flint being installed.

### Contents

The contents should be a repository with a single version of a single package, packed (NOT compressed) into a tar archive.

### On disk format

Headers may be 512 KB, 1024 KB, or larger in 512 KB increments. The end of the header is identified by the bytes `75 73 74 61 72` (`ustar` in ASCII, the standard tar file signature). This allows for flexible header sizes.
