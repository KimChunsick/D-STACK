## Needed info

H1. The current latest stable source release of Git is `2.55.0`, released on `2026-06-29`. Supported by S1, S2, S3.

H2. GitHub’s `git/git` tag list shows `v2.55.0` as the newest non-rc tag, dated `Jun 29, 2026`, with later nearby entries only being release candidates for the same version. Supported by S2.

H3. Kernel.org’s Git source tarball directory contains `git-2.55.0.tar.gz`, `.tar.xz`, and `.tar.sign`, all timestamped `29-Jun-2026 16:55`, and no `git-2.56` entry was found in the same index. Supported by S3.

Data-check ledger:

| H-id | source (URL + version/date) | unit | denominator | transformation | value | status | how sure |
|---|---|---:|---:|---|---|---|---|
| H1 | S1, git-scm.com homepage crawled 2026-08-21 | version/date | N/A | quoted latest source release field | `2.55.0`, `2026-06-29` | quoted | high |
| H2 | S2, GitHub tags page crawled 2026-08-21 | tag/date | N/A | compared visible newest stable tag against rc tags | `v2.55.0`, `Jun 29, 2026` | quoted | high |
| H3 | S3, Kernel.org mirror index crawled 2026-08-19 / opened 2026-08-21 | tarball timestamp | N/A | checked `2.55.0` entries and searched for `git-2.56` | `git-2.55.0`, `29-Jun-2026 16:55`; no `git-2.56` found | quoted | medium-high |

Deferred executable checks: none.

## Opposing views

The main ambiguity is packaging versus upstream source release. Git for Windows shows `2.55.0(3)` released on `2026-07-14`, which is newer by date but is a Windows distribution build, not the upstream Git source release. It also states that the current source code release is `2.55.0`. This supports treating `2.55.0`, released `2026-06-29`, as the answer for “Git version-control system” rather than “Git for Windows.” Source: S4.

## For the goal

Primary upstream-facing sources agree: the official Git site lists “Latest source release 2.55.0” with date `2026-06-29`; GitHub’s `git/git` tags show `v2.55.0` dated `Jun 29, 2026`; Kernel.org’s Git source archive lists `git-2.55.0` tarballs timestamped `29-Jun-2026`.

## Against the goal

A single “current latest stable release” can be misleading if the user means a platform package rather than upstream source. For example, Git for Windows has a maintained build `2.55.0(3)` released `2026-07-14`, but that does not change the upstream Git version number. I found no primary-source evidence in the checked sources for a newer upstream stable Git release such as `2.56.0`.

## Unverified

I did not independently clone the upstream Git repository or verify signed tags locally. I relied on primary web sources: git-scm.com, GitHub’s `git/git` tag page, and Kernel.org’s source tarball index.

## Sources

S1. Official Git homepage, “Latest source release”: `2.55.0` with date `2026-06-29`. https://git-scm.com/ Accessed/crawled `2026-08-21`.

S2. GitHub `git/git` tags page: `v2.55.0`, signed tag by Junio C Hamano, dated `Jun 29, 2026`; rc tags appear below it. https://github.com/git/git/tags Accessed/crawled `2026-08-21`.

S3. Kernel.org mirror Git source archive: `git-2.55.0.tar.gz`, `.tar.sign`, `.tar.xz` timestamped `29-Jun-2026 16:55`; search found no `git-2.56` entry. https://mirrors.edge.kernel.org/pub/software/scm/git/ Accessed `2026-08-21`; page crawled `2026-08-19`.

S4. Official Git install page for Windows: latest Git for Windows build `2.55.0(3)` released `2026-07-14`, while stating current source code release is `2.55.0`. https://git-scm.com/install/windows?pubDate=20260611 Accessed via search result `2026-08-21`.