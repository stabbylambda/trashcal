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
import * as s3 from "aws-cdk-lib/aws-s3";

export interface TrashcalCdkStackProps extends cdk.StackProps {
  domainName: string;
  email: string;
  cert: acm.Certificate;
}
export class TrashcalCdkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: TrashcalCdkStackProps) {
    super(scope, id, props);

    // The GitHub OIDC provider is an account-level singleton, created in a
    // prior deploy. Reference the existing one rather than managing it here.
    const provider = GithubActionsIdentityProvider.fromAccount(
      this,
      "GithubProvider",
    );
    new GithubActionsRole(this, "UploadRole", {
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

    // Create the log group
    const logGroup = new logs.LogGroup(this, "trashcal-logs", {
      retention: logs.RetentionDays.ONE_MONTH,
    });

    // Create the rust lambda
    const trashcal = new RustFunction(this, "trashcal-lambda", {
      entry: "../lambda",
      architecture: Architecture.ARM_64,
      logGroup,
      environment: {
        AWS_LAMBDA_LOG_FORMAT: "json",
      },
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

    // CloudFront access logs bucket. Standard (legacy) CloudFront logging
    // delivers log files using bucket ACLs, so ACLs must stay enabled
    // (BUCKET_OWNER_PREFERRED) and the bucket can't use SSE-KMS — hence
    // SSE-S3. The 90-day lifecycle rule caps growth. This replaces the old
    // out-of-band `trashcal-access-logs` bucket, which can be decommissioned
    // once this distribution is delivering logs here.
    const accessLogsBucket = new s3.Bucket(this, "trashcal-access-logs", {
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      objectOwnership: s3.ObjectOwnership.BUCKET_OWNER_PREFERRED,
      encryption: s3.BucketEncryption.S3_MANAGED,
      enforceSSL: true,
      lifecycleRules: [
        {
          id: "expire-access-logs",
          expiration: cdk.Duration.days(90),
        },
      ],
    });

    new cloudfront.Distribution(this, "cloudfront-api", {
      domainNames: [props.domainName],
      enableLogging: true,
      logBucket: accessLogsBucket,
      logFilePrefix: "cloudfront/",
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
