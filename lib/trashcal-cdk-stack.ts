import * as dotenv from "dotenv";
dotenv.config();
import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import { RustFunction, Settings } from "rust.aws-cdk-lambda";
import * as apigwv2 from "@aws-cdk/aws-apigatewayv2-alpha";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import { HttpLambdaIntegration } from "@aws-cdk/aws-apigatewayv2-integrations-alpha";

if (!process.env.DOMAIN_NAME) {
  console.log("DOMAIN_NAME environment variable is not set");
}

const domainName = process.env.DOMAIN_NAME ?? process.exit(1);

export class TrashcalCdkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const trashcal = new RustFunction(this, "trashcal", {
      directory: "./lambda",
    });

    const trashcalIntegration = new HttpLambdaIntegration(
      "TrashcalIntegration",
      trashcal
    );

    const cert = new acm.Certificate(this, "trashcal-cert", {
      domainName,
      // I'm using Cloudflare as my DNS provider, so this part requires going into the certificate
      // manager and creating the CNAME record that AWS wants.
      validation: acm.CertificateValidation.fromDns(),
    });

    const domain = new apigwv2.DomainName(this, domainName, {
      domainName,
      certificate: cert,
    });

    const api = new apigwv2.HttpApi(this, "trashcal-api", {
      disableExecuteApiEndpoint: true,
      defaultDomainMapping: {
        domainName: domain,
      },
    });

    api.addRoutes({
      path: "/{id}",
      methods: [apigwv2.HttpMethod.GET],
      integration: trashcalIntegration,
    });
  }
}
