# DnsResolutionRequest

Request to resolve domains

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**leech_uuid** | **str** | The leech to use  Leave empty to use a random leech | [optional] 
**targets** | **List[str]** | The domains to resolve | 
**concurrent_limit** | **int** | The concurrent task limit | 
**workspace_uuid** | **str** | The workspace to execute the attack in | 

## Example

```python
from kraken_sdk.models.dns_resolution_request import DnsResolutionRequest

# TODO update the JSON string below
json = "{}"
# create an instance of DnsResolutionRequest from a JSON string
dns_resolution_request_instance = DnsResolutionRequest.from_json(json)
# print the JSON string representation of the object
print DnsResolutionRequest.to_json()

# convert the object into a dict
dns_resolution_request_dict = dns_resolution_request_instance.to_dict()
# create an instance of DnsResolutionRequest from a dict
dns_resolution_request_form_dict = dns_resolution_request.from_dict(dns_resolution_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


