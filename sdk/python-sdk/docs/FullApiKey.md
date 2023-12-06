# FullApiKey

A representation of a full api key

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The key&#39;s identifier | 
**name** | **str** | A descriptive name helping the user to identify the key | 
**key** | **str** | The actual key&#39;s value | 

## Example

```python
from kraken_sdk.models.full_api_key import FullApiKey

# TODO update the JSON string below
json = "{}"
# create an instance of FullApiKey from a JSON string
full_api_key_instance = FullApiKey.from_json(json)
# print the JSON string representation of the object
print FullApiKey.to_json()

# convert the object into a dict
full_api_key_dict = full_api_key_instance.to_dict()
# create an instance of FullApiKey from a dict
full_api_key_form_dict = full_api_key.from_dict(full_api_key_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


