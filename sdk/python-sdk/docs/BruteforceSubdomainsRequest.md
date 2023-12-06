# BruteforceSubdomainsRequest

The settings of a subdomain bruteforce request

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**leech_uuid** | **str** | The leech to use  Leave empty to use a random leech | [optional] 
**domain** | **str** | Domain to construct subdomains for | 
**wordlist_uuid** | **str** | The wordlist to use | 
**concurrent_limit** | **int** | The concurrent task limit | 
**workspace_uuid** | **str** | The workspace to execute the attack in | 

## Example

```python
from kraken_sdk.models.bruteforce_subdomains_request import BruteforceSubdomainsRequest

# TODO update the JSON string below
json = "{}"
# create an instance of BruteforceSubdomainsRequest from a JSON string
bruteforce_subdomains_request_instance = BruteforceSubdomainsRequest.from_json(json)
# print the JSON string representation of the object
print BruteforceSubdomainsRequest.to_json()

# convert the object into a dict
bruteforce_subdomains_request_dict = bruteforce_subdomains_request_instance.to_dict()
# create an instance of BruteforceSubdomainsRequest from a dict
bruteforce_subdomains_request_form_dict = bruteforce_subdomains_request.from_dict(bruteforce_subdomains_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


