#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { TrashcalCdkStack } from "../lib/trashcal-cdk-stack";
import { checkInstalledTarget } from "@cdklabs/aws-lambda-rust/lib/util";

console.log(
  "Target installed?",
  checkInstalledTarget("aarch64-unknown-linux-gnu")
);

const app = new cdk.App();
new TrashcalCdkStack(app, "TrashcalCdkStack", {});
