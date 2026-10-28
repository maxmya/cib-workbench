<!-- SPDX-License-Identifier: MIT -->

# FEAT-179 — A call's rules and answers, from its code, and OpenAPI out

**Status:** Done — in the working tree on `release/1.1.0`, uncommitted. Covered by `import/java.rs`, `import/nest.rs`, `import/contract.rs`, `import/agent.rs` and `calls/openapi.rs`, and by `CallDocs.test.ts`, `ShareDialog.test.ts`, `ImportSheet.test.ts` › *asking an agent too* and `CallsScreen.test.ts`. Nothing of it has been looked at in the running application, and the agent's read has not been tried with a real agent.
**Branch:** none yet.
**Screens:** Calls (2C), Import, Share.
**Raised by:** the author, 2026-10-28: "when scan code we should make the algorithm that extracts endpoints from controllers also extract it's body and fill example of it's validations , scans if this endpoint returns errors or any other thing ( like how it response on 200 and 400 and 500 etc .. ) we can make this algorithm + enhance it with agent (optional) … [ all should be exported to a standard format that can be imported on other systems ]".

## Change

- **Rules.** A scanned call carries its body's field rules: type, required, format, bounds, length, pattern and allowed values. For Spring and JAX-RS they come from Bean Validation; for NestJS from class-validator. The body's example respects them.
- **Answers.** A scanned call carries what it answers, a status at a time (200/201/204, 400, 401/403, 404, 409, 500), each with an example body and where it was read from: the return type, `@ResponseStatus`, a thrown exception and its handler, `@ApiResponse`, Nest's exceptions or the swagger decorators. An exception thrown by an injected service is followed one call deep.
- **Docs.** A call with rules or answers has a *Docs* tab that lists the fields with their rules in words and each answer with its example. A scan reads them again.
- **An agent, if wanted.** Import's Pick step offers to have an agent the repository allows read the chosen controllers. It is given each file in the question and runs nothing. What it adds goes in beside what the scan read and never replaces it, and each answer it adds names it.
- **OpenAPI out.** Share has *OpenAPI 3.1* and *OpenAPI YAML* as one-file formats, and each collection's `⋯` has *Export as OpenAPI…*. Rules become the body's schema and answers become its responses. The server is an environment's `baseUrl` unless that is secret. SOAP calls are left out.

## Acceptance criteria

- A Spring or Nest controller's call shows its rules and answers in Docs.
- The OpenAPI file opens in Swagger UI and other tools with them.
- The agent's read is optional, and adds without overwriting.
