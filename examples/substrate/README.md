# 'Substrate' example template

* uses the substrate helper functions to allow for signatures on substrate chains

* Creates its AuxData and Config with mandatory fields then passes them to ``check_message_against_transaction`` 
* Once done can now use AuxData fields to apply constraints

### Not checked 
* currently nonce and block mortality are not checked (will be addressed eventually)


