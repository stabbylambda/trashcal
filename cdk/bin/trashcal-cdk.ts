#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { TrashcalCdkStack } from "../lib/trashcal-cdk-stack";

const app = new cdk.App();
new TrashcalCdkStack(app, "TrashcalCdkStack", {});
