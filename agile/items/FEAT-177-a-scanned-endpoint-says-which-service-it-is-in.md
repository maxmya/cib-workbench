<!-- SPDX-License-Identifier: MIT -->

# FEAT-177 — A scanned endpoint says which service it is in

**Status:** Done — in the working tree on `release/1.1.0`, uncommitted. Covered by `calls/import/service.rs` and `import/mod.rs` › *controllers of two services are brought in under their service*.
**Branch:** none yet.
**Screens:** Calls (2C), Import.
**Raised by:** the author, 2026-10-28: "as we separate calls of each controller we shall say which service has those calls ( microservice or nestjs app )".

## Change

- A scan finds the services in the repository: each Maven module, Gradle build, `package.json` and NestJS monorepo app, named by its artifact, settings or package, and says what kind it is (Spring Boot, NestJS).
- An endpoint is in the service whose root is nearest above its file. Import shows controllers under their service, and with more than one service a collection nests each service's controllers in a folder of its own.

## Acceptance criteria

- With two services, Import and the collection say which endpoints are whose.
