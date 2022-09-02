Aptos Node Deployment
=========================

This directory provides Terraform modules for a typical Aptos Node deployment, which includes both a validator node and fullnode, as well as HAProxy so that it's easy to manage incoming traffic. 

These Terraform modules are cloud-specific, and generally consist of a few high-level components:
* Cloud network configuration
* An installation of that cloud's managed Kubernetes service
* [Helm](https://helm.sh/) releases into that kubernetes cluster

If you wish to deploy an Aptos Node from scratch, Terraform is an easy way to spin that up on a public cloud. Alternatively, you may install the Helm charts directly on pre-existing Kubernetes clusters.

Steps:
1. Install prerequisites
2. Set up your account
3. Set up your remote state file
4. Terraform apply
5. Check it
6. To operate, see the helm/aptos-node README.md
