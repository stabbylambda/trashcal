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
        // Managed policy (no custom cache policy / no access logging) keeps us on
        // CloudFront's free plan. CACHING_OPTIMIZED is the one that works here: it
        // keeps NO request headers in the cache key. That matters because the API
        // Gateway origin routes by Host and has no custom domain, so the viewer Host
        // must not be forwarded (hence ALL_VIEWER_EXCEPT_HOST_HEADER above). The
        // UseOriginCacheControlHeaders / Amplify policies all force Host into the
        // cache key, which makes CloudFront forward the viewer Host and API Gateway
        // answer 403 Forbidden. Format is selected by URL suffix, so the path —
        // always in the cache key — keeps JSON and iCal separate. TTL follows the
        // Expires header the Lambda emits.
        cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
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
