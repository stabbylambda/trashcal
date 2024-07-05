import * as dotenv from "dotenv";
dotenv.config();
import * as cdk from "aws-cdk-lib";
import * as iam from "aws-cdk-lib/aws-iam";
import { Construct } from "constructs";
import { RustFunction } from "@cdklabs/aws-lambda-rust";
import * as apigwv2 from "aws-cdk-lib/aws-apigatewayv2";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import { HttpLambdaIntegration } from "aws-cdk-lib/aws-apigatewayv2-integrations";
import * as logs from "aws-cdk-lib/aws-logs";
import * as sns from "aws-cdk-lib/aws-sns";
import * as subscriptions from "aws-cdk-lib/aws-sns-subscriptions";
import { Alarm, TreatMissingData } from "aws-cdk-lib/aws-cloudwatch";
import { SnsAction } from "aws-cdk-lib/aws-cloudwatch-actions";
import { Architecture } from "aws-cdk-lib/aws-lambda";
import {
  GithubActionsIdentityProvider,
  GithubActionsRole,
} from "aws-cdk-github-oidc";
import * as cloudfront from "aws-cdk-lib/aws-cloudfront";
import * as cloudfrontOrigins from "aws-cdk-lib/aws-cloudfront-origins";

export interface TrashcalCdkStackProps extends cdk.StackProps {
  domainName: string;
  email: string;
  cert: acm.Certificate;
}
export class TrashcalCdkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: TrashcalCdkStackProps) {
    super(scope, id, props);

    const provider = new GithubActionsIdentityProvider(this, "GithubProvider");
    const uploadRole = new GithubActionsRole(this, "UploadRole", {
      provider: provider,
      owner: "stabbylambda",
      repo: "trashcal",
      filter: "ref:refs/heads/main",
      inlinePolicies: {
        CdkDeploymentPolicy: new iam.PolicyDocument({
          assignSids: true,
          statements: [
            new iam.PolicyStatement({
              effect: iam.Effect.ALLOW,
              actions: ["sts:AssumeRole"],
              resources: [`arn:aws:iam::${this.account}:role/cdk-*`],
            }),
          ],
        }),
      },
    });

    // Create the rust lambda
    const trashcal = new RustFunction(this, "trashcal-lambda", {
      entry: "../trashcal-lambda",
      architecture: Architecture.ARM_64,
      logRetention: logs.RetentionDays.ONE_MONTH,
      environment: {
        AWS_LAMBDA_LOG_FORMAT: "json",
      },
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

    // create the api gateway endpoint
    const api = new apigwv2.HttpApi(this, "trashcal-api", {});

    // only one route
    api.addRoutes({
      path: "/{id}",
      methods: [apigwv2.HttpMethod.GET],
      integration: trashcalIntegration,
    });

    let cachePolicy = new cloudfront.CachePolicy(
      this,
      "trashcal-cache-policy",
      {
        minTtl: cdk.Duration.hours(1),
        maxTtl: cdk.Duration.days(2),
        defaultTtl: cdk.Duration.days(1),
        enableAcceptEncodingBrotli: true,
        enableAcceptEncodingGzip: true,
        headerBehavior: cloudfront.CacheHeaderBehavior.allowList(
          // we need to add the Accept header to the cache key because otherwise json and ical collide
          "Accept"
        ),
      }
    );

    new cloudfront.Distribution(this, "cloudfront-api", {
      domainNames: [props.domainName],
      defaultBehavior: {
        origin: new cloudfrontOrigins.HttpOrigin(
          `${api.apiId}.execute-api.${cdk.Stack.of(this).region}.amazonaws.com`
        ),
        viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
        allowedMethods: cloudfront.AllowedMethods.ALLOW_ALL,

        originRequestPolicy:
          cloudfront.OriginRequestPolicy.ALL_VIEWER_EXCEPT_HOST_HEADER,
        cachePolicy: cachePolicy,
      },
      certificate: props.cert,
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
    topic.addSubscription(new subscriptions.EmailSubscription(props.email));
    panicAlarm.addAlarmAction(new SnsAction(topic));
  }
}
