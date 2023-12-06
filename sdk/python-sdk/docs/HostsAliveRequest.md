# HostsAliveRequest

Host Alive check request

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**leech_uuid** | **str** | The leech to use  Leave empty to use a random leech | [optional] 
**targets** | **List[str]** | The ip addresses / networks to scan | 
**timeout** | **int** | The time to wait until a host is considered down.  The timeout is specified in milliseconds. | 
**concurrent_limit** | **int** | The concurrent task limit | 
**workspace_uuid** | **str** | The workspace to execute the attack in | 

## Example

```python
from kraken_sdk.models.hosts_alive_request import HostsAliveRequest

# TODO update the JSON string below
json = "{}"
# create an instance of HostsAliveRequest from a JSON string
hosts_alive_request_instance = HostsAliveRequest.from_json(json)
# print the JSON string representation of the object
print HostsAliveRequest.to_json()

# convert the object into a dict
hosts_alive_request_dict = hosts_alive_request_instance.to_dict()
# create an instance of HostsAliveRequest from a dict
hosts_alive_request_form_dict = hosts_alive_request.from_dict(hosts_alive_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


