## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: The repository publishes a supported vulnerability reporting path
The repository SHALL include a tracked security policy that tells reporters how to submit vulnerabilities without using public issues.

#### Scenario: Reporter looks for security contact guidance
- **WHEN** a user opens the repository's security policy
- **THEN** they are instructed to use GitHub private vulnerability reporting
- **THEN** they are told not to disclose exploitable details in public issues

### Requirement: The repository monitors dependency risk for public OSS operation
The repository SHALL enable GitHub dependency monitoring features appropriate for a public repository baseline.

#### Scenario: Vulnerable dependency is identified
- **WHEN** the dependency graph detects a vulnerable dependency in the repository
- **THEN** a Dependabot alert is available in the repository security view
- **THEN** a security update path exists when GitHub can propose a fix

### Requirement: The repository performs automated code and workflow security checks
The repository SHALL enable security automation for code and dependency changes before broad OSS collaboration.

#### Scenario: Supported source languages are present
- **WHEN** the public repository contains a CodeQL-supported language
- **THEN** code scanning is enabled through GitHub's default setup or an equivalent maintained configuration

#### Scenario: Pull request adds or updates dependencies
- **WHEN** a pull request changes dependency manifests, lockfiles, or workflow dependencies
- **THEN** the repository runs dependency or workflow security review before merge

### Requirement: Public contribution paths are protected by repository rules
The repository SHALL protect its default branch with pull-request based changes and required checks.

#### Scenario: Contributor proposes a change to the default branch
- **WHEN** a contributor opens a pull request against the default branch
- **THEN** the change is evaluated under repository rules that block force-push style bypass of review
- **THEN** required status checks must pass before the change is merged

### Requirement: GitHub Actions use least privilege and trusted dependencies
The repository MUST harden GitHub Actions for public collaboration.

#### Scenario: Workflow runs on repository automation
- **WHEN** a GitHub Actions workflow executes
- **THEN** the workflow uses restricted `GITHUB_TOKEN` permissions by default
- **THEN** third-party actions are pinned to immutable revisions or come from explicitly trusted sources

#### Scenario: External fork triggers a workflow
- **WHEN** a public fork pull request would run repository workflows
- **THEN** the repository requires maintainer approval according to the configured fork workflow approval policy
