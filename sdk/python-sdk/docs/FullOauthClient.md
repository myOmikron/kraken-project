# FullOauthClient

A complete version of a workspace

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**name** | **str** |  | 
**redirect_uri** | **str** |  | 
**secret** | **str** |  | 

## Example

```python
from kraken_sdk.models.full_oauth_client import FullOauthClient

# TODO update the JSON string below
json = "{}"
# create an instance of FullOauthClient from a JSON string
full_oauth_client_instance = FullOauthClient.from_json(json)
# print the JSON string representation of the object
print FullOauthClient.to_json()

# convert the object into a dict
full_oauth_client_dict = full_oauth_client_instance.to_dict()
# create an instance of FullOauthClient from a dict
full_oauth_client_form_dict = full_oauth_client.from_dict(full_oauth_client_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


