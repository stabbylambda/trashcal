import * as dotenv from "dotenv";
dotenv.config();
import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import { PackageManagerType, RustFunction } from "@cdklabs/aws-lambda-rust";
import * as apigwv2 from "aws-cdk-lib/aws-apigatewayv2";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import { HttpLambdaIntegration } from "aws-cdk-lib/aws-apigatewayv2-integrations";
import * as logs from "aws-cdk-lib/aws-logs";
import * as sns from "aws-cdk-lib/aws-sns";
import * as subscriptions from "aws-cdk-lib/aws-sns-subscriptions";
import { Alarm, Metric, TreatMissingData } from "aws-cdk-lib/aws-cloudwatch";
import { SnsAction } from "aws-cdk-lib/aws-cloudwatch-actions";
import { Architecture } from "aws-cdk-lib/aws-lambda";

// Load up info from .env file
const domainName = getEnv("DOMAIN_NAME");
const email = getEnv("EMAIL");

export class TrashcalCdkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // Create the rust lambda
    const trashcal = new RustFunction(this, "trashcal-lambda", {
      entry: "../trashcal-lambda",
      bundling: {
        packageManagerType: PackageManagerType.CARGO_ZIGBUILD,
      },
      architecture: Architecture.ARM_64,
      logRetention: logs.RetentionDays.ONE_MONTH,
    });

    // Create the log group
    const logGroup = new logs.LogGroup(this, "trashcal-logs", {
      logGroupName: `/aws/lambda/${trashcal.functionName}`,
      retention: logs.RetentionDays.ONE_MONTH,
    });

    const trashcalIntegration = new HttpLambdaIntegration(
      "trashcal-integration",
      trashcal
    );

    // set up the https cert and domain
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

    // create the api gateway endpoint
    const api = new apigwv2.HttpApi(this, "trashcal-api", {
      disableExecuteApiEndpoint: true,
      defaultDomainMapping: {
        domainName: domain,
      },
    });

    // only one route
    api.addRoutes({
      path: "/{id}",
      methods: [apigwv2.HttpMethod.GET],
      integration: trashcalIntegration,
    });

    const metricNamespace = "trashcal";

    // create a metric for counting total calendars
    new logs.MetricFilter(this, "trashcal-total", {
      logGroup,
      metricNamespace,
      metricName: "total",
      filterPattern: logs.FilterPattern.allTerms("Returning calendar"),
      metricValue: "1",
    });

    // create a metric for rust panics (hi Paul! 👋)
    const panicFilter = new logs.MetricFilter(
      this,
      "trashcal-panic-metric-filter",
      {
        logGroup,
        metricNamespace,
        metricName: "panics",
        filterPattern: logs.FilterPattern.allTerms("panicked"),
        metricValue: "1",
      }
    );

    // if rust panicks at all, set the alarm
    const panicAlarm = new Alarm(this, "trashcal-panic-alarm", {
      metric: panicFilter.metric(),
      evaluationPeriods: 1,
      threshold: 1,
      treatMissingData: TreatMissingData.NOT_BREACHING,
      actionsEnabled: true,
    });

    const topic = new sns.Topic(this, "trashcal-panics");
    topic.addSubscription(new subscriptions.EmailSubscription(email));
    panicAlarm.addAlarmAction(new SnsAction(topic));
  }
}

function getEnv(name: string): string {
  if (!process.env[name]) {
    console.log(`${name} environment variable is not set`);
  }

  return process.env[name] ?? process.exit(1);
}
