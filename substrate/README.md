# `entropy-programs-substrate`

Provides substrate helper functions to help with talking to substrate chains

See examples/substrate for example of usage

### Usage

* Takes a Config and SignatureRequest
    * Uses this data to build a transaction and use the data from the aux info to rebuild a transaction and compare it to what a user it trying to sign 
    * Errors if they don't match
    * After this a program dev can use the details in the transaction to apply constraints, knowing that that is what is being signed
