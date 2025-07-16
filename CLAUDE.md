# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Trashcal is an AWS Lambda function that parses the City of San Diego's trash collection pages and produces iCal files with events for trash, recycling, and greens days. The project consists of:

- **Rust Lambda function** (`trashcal-lambda/`): Core logic for parsing San Diego's Salesforce app and generating iCal calendars
- **AWS CDK infrastructure** (`cdk/`): TypeScript CDK code for deploying the Lambda, API Gateway, CloudFront distribution, and monitoring

## Architecture

### Lambda Function (Rust)
- Entry point: `trashcal-lambda/src/main.rs`
- Core logic: `trashcal-lambda/src/trashcal.rs` - handles the main calendar generation
- Pickup logic: `trashcal-lambda/src/pickup.rs` and `pickup_calendar.rs` - parses San Diego's HTML pages
- Error handling: `trashcal-lambda/src/error.rs`

### CDK Infrastructure (TypeScript)
- Main stack: `cdk/lib/trashcal-cdk-stack.ts` - defines Lambda, API Gateway, CloudFront, and monitoring
- Certificate stack: `cdk/lib/trashcal-cert-stack.ts` - separate stack for SSL certificates
- Entry point: `cdk/bin/trashcal-cdk.ts`

The Lambda function scrapes HTML from San Diego's "bookmarkable page" at `getitdone.force.com/CollectionDetail` to extract pickup schedules, then generates iCal format calendars.

## Development Commands

### CDK (TypeScript)
```bash
cd cdk
npm run cdk <command>           # Run any CDK command
npm run deploy                  # Deploy all stacks and run smoke tests
npm run smoke                   # Run smoke tests using vitest
```

### Rust Lambda
The Rust code requires [cargo lambda](https://www.cargo-lambda.info) for building:
```bash
cd trashcal-lambda
cargo lambda build              # Build the Lambda function
cargo test                      # Run unit tests
cargo test --test integration_tests  # Run integration tests
```

### Testing
- CDK tests: Uses Jest (see `cdk/jest.config.ts`)
- Rust tests: Standard `cargo test` with integration tests in `tests/` directory
- Smoke tests: Uses vitest to test deployed endpoints
- Test data: Located in `trashcal-lambda/tests/data/` with various JSON fixtures

## Key Configuration

### Environment Requirements
- Set `DOMAIN_NAME` environment variable for CDK deployment
- AWS credentials configured for CDK deployment
- Rust toolchain and cargo-lambda for Lambda development

### CDK Context
The CDK app uses extensive AWS feature flags in `cdk/cdk.json` to enable modern CDK behaviors.

### Lambda Configuration
- Architecture: ARM64
- Runtime: Rust with custom runtime
- Log format: JSON
- Log retention: 1 month
- Caching: 1-hour to 2-day TTL via CloudFront with Accept header in cache key

## Monitoring and Observability

The CDK stack includes CloudWatch monitoring:
- **Total calendars metric**: Counts "Returning calendar" log entries
- **Panic alarm**: Monitors for Rust panics and sends SNS notifications
- **Log retention**: 1 month for Lambda logs

## Integration Tests

The `trashcal-lambda/tests/integration_tests.rs` file contains tests that verify parsing of real San Diego data formats. Test fixtures in `tests/data/` cover various scenarios like path-based vs query-based URLs, different calendar formats, and error cases.