# UpdateApiKeyRequest

The request to update an api key

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** | A descriptive name helping the user to identify the key | 

## Example

```python
from kraken_sdk.models.update_api_key_request import UpdateApiKeyRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateApiKeyRequest from a JSON string
update_api_key_request_instance = UpdateApiKeyRequest.from_json(json)
# print the JSON string representation of the object
print UpdateApiKeyRequest.to_json()

# convert the object into a dict
update_api_key_request_dict = update_api_key_request_instance.to_dict()
# create an instance of UpdateApiKeyRequest from a dict
update_api_key_request_form_dict = update_api_key_request.from_dict(update_api_key_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


