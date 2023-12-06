# SimpleOauthClient

A simple (secret-less) version of a workspace

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**name** | **str** |  | 
**redirect_uri** | **str** |  | 

## Example

```python
from kraken_sdk.models.simple_oauth_client import SimpleOauthClient

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleOauthClient from a JSON string
simple_oauth_client_instance = SimpleOauthClient.from_json(json)
# print the JSON string representation of the object
print SimpleOauthClient.to_json()

# convert the object into a dict
simple_oauth_client_dict = simple_oauth_client_instance.to_dict()
# create an instance of SimpleOauthClient from a dict
simple_oauth_client_form_dict = simple_oauth_client.from_dict(simple_oauth_client_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


