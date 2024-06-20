#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { TrashcalCdkStack } from "../lib/trashcal-cdk-stack";
import { spawnSync } from "child_process";

const proc = spawnSync("rustup", ["target", "list", "--installed"]);
console.log(proc);
console.log(proc.stdout.toString());

const app = new cdk.App();
new TrashcalCdkStack(app, "TrashcalCdkStack", {});
