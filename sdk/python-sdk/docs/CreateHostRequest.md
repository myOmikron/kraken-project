# CreateHostRequest

The request to manually add a host

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ip_addr** | **str** | The host&#39;s ip address | 
**certainty** | [**ManualHostCertainty**](ManualHostCertainty.md) |  | 

## Example

```python
from kraken_sdk.models.create_host_request import CreateHostRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateHostRequest from a JSON string
create_host_request_instance = CreateHostRequest.from_json(json)
# print the JSON string representation of the object
print CreateHostRequest.to_json()

# convert the object into a dict
create_host_request_dict = create_host_request_instance.to_dict()
# create an instance of CreateHostRequest from a dict
create_host_request_form_dict = create_host_request.from_dict(create_host_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


