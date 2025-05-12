# accelerate-bootcamp

To be able to get approval for [Scale or Die](https://solana.com/accelerate#tickets), you'll need to successful complete this bootcamp in person in the NYC office.

There will be two 2-day sessions for this bootcamp, 5/12 - 5/13 and 5/14 - 5/15. Please make sure you attend the first and second day of one.

# Pre-requisites

### This entire section must be completed BEFORE Day 1.

1. Install Solana tool suite:

   ```shell
   npx mucho install
   ```

   or follow the installation docs [here](https://solana.com/docs/intro/installation).

2. Create an open sourced repository titled "Solana Accelerate".

3. Fill out this [google form](https://docs.google.com/forms/d/e/1FAIpQLSeC4fc-gmzOKepmkDic7RHw2mWwwJR1BTz_Ru8CfmmpcznAPw/viewform?usp=dialog).

4. Set up your devnet solana wallet and fund it with SOL. Use the faucet [here](https://faucet.solana.com/).

5. Create a new directory in the Accelerate Bootcamp repo titled `project-1-crud-app`. Follow this [guide](https://solana.com/developers/guides/dapps/journal) and then create, build and deploy to devnet with the address you provided in the above google form.

   (For more guidance, here is a [video tutorial](https://youtu.be/VRCcUlUTjfc?si=0YmkvD_FWH1DnvdH))

# Day One

## Creating an anchor client

`project-2-anchor-client` -

There has been a solana program deployed on devnet with this address: `5Couhd2qWo7v3L8LR3Q4daDPdFyJpV8MNqi3wkzNWGvu`

_(Note: the address will not be shown until day 1 in the office)_

The Solana program allows you to vote on a given topic.

A front end was not created for this program so you will need to create an anchor client to enable to vote.

Make sure you cast your vote with the devnet address you provided in the above google form.

For reference, [here](https://github.com/solana-developers/developer-bootcamp-2024/tree/main/project-1-favorites) is the source code for the favorites voting app

## Accounts and PDAs

`project-3-pdas`-

Understanding the [Solana Account model](https://solana.com/docs/core/accounts) and [Program Derived Addresses (PDAs)](https://solana.com/docs/core/pda) is key to Solana program development.

Update the journal CRUD app to better utilize Program Derived Addresses (PDAs), optimizing for frontend indexing. The goal is to enable efficient retrieval and display of all journal entries for a given user.

Hint: Create this project with just `anchor init`, a full dApp is not needed here.

## CPIs

`project-4-cpis` -

Understanding [Cross Program Innovations (CPIs)](https://solana.com/docs/core/cpi) and [Token Accounts](https://solana.com/docs/tokens/basics) is another key to Solana program development.

Create an app that creates token accounts, mints tokens, and transfers tokens.

# Day Two

`project-5-capstone` -

Create your own Solana project that must incorporate at least two of the following topics:

- Pyth
- Switchboard
- Associated Token Accounts (ATAs)
- Token Accounts
- At least 2 different PDAs in one instruction
- At least 2 different CPIs in one instruction
- Token Extensions
- Blinks
- Compressed NFTs

Tests must be written with either:

- Bankrun
- LiteSVM
- Solana-program-test

Once completed, build and deploy your program to devnet :)

# Resources

Here is a list of helpful resources when building on Solana:

1. [Solana Stack Exchange](https://solana.stackexchange.com/)
2. [Devnet Faucet](https://faucet.solana.com/)
3. [Program Examples](https://github.com/solana-developers/program-examples)
4. [Block explorer](https://solscan.io/)
5. [Developer Docs](https://solana.com/docs)
6. [Anchor Docs](https://www.anchor-lang.com/docs)
7. [Solana Developer Bootcamp](https://github.com/solana-developers/developer-bootcamp-2024)
8. [Toolkit](https://solana.com/docs/toolkit)
