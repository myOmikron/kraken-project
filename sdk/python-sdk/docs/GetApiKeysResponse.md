# GetApiKeysResponse

The response that contains all api keys

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**keys** | [**List[FullApiKey]**](FullApiKey.md) |  | 

## Example

```python
from kraken_sdk.models.get_api_keys_response import GetApiKeysResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetApiKeysResponse from a JSON string
get_api_keys_response_instance = GetApiKeysResponse.from_json(json)
# print the JSON string representation of the object
print GetApiKeysResponse.to_json()

# convert the object into a dict
get_api_keys_response_dict = get_api_keys_response_instance.to_dict()
# create an instance of GetApiKeysResponse from a dict
get_api_keys_response_form_dict = get_api_keys_response.from_dict(get_api_keys_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


