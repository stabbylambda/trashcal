import "source-map-support/register";

import * as cdk from "aws-cdk-lib";
import { TrashcalCdkStack } from "../lib/trashcal-cdk-stack";
import { TrashcalCertStack } from "../lib/trashcal-cert-stack";
import { getEnv } from "./util";

// Load up info from .env file
const domainName = getEnv("DOMAIN_NAME");
const email = getEnv("EMAIL");

const app = new cdk.App();
// The cert *must* be depoloyed in us-east-1 because of Cloudfront
let certStack = new TrashcalCertStack(app, "TrashcalCertStack", {
  crossRegionReferences: true,
  domainName,
  env: {
    region: "us-east-1",
  },
});

new TrashcalCdkStack(app, "TrashcalCdkStack", {
  crossRegionReferences: true,
  domainName,
  email,
  env: {
    region: "us-west-2",
  },
  cert: certStack.cert,
});
