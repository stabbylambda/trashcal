import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import * as acm from "aws-cdk-lib/aws-certificatemanager";

export interface TrashcalCertStackProps extends cdk.StackProps {
  domainName: string;
}

export class TrashcalCertStack extends cdk.Stack {
  public cert: acm.Certificate;

  constructor(scope: Construct, id: string, props: TrashcalCertStackProps) {
    super(scope, id, props);

    // set up the https cert and domain
    this.cert = new acm.Certificate(this, "trashcal-cert", {
      domainName: props.domainName,
      // I'm using Cloudflare as my DNS provider, so this part requires going into the certificate
      // manager and creating the CNAME record that AWS wants.
      validation: acm.CertificateValidation.fromDns(),
    });
  }
}
