# OpenRequestInfo

The information about an oauth request

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**workspace** | [**SimpleWorkspace**](SimpleWorkspace.md) |  | 
**oauth_application** | [**SimpleOauthClient**](SimpleOauthClient.md) |  | 

## Example

```python
from kraken_sdk.models.open_request_info import OpenRequestInfo

# TODO update the JSON string below
json = "{}"
# create an instance of OpenRequestInfo from a JSON string
open_request_info_instance = OpenRequestInfo.from_json(json)
# print the JSON string representation of the object
print OpenRequestInfo.to_json()

# convert the object into a dict
open_request_info_dict = open_request_info_instance.to_dict()
# create an instance of OpenRequestInfo from a dict
open_request_info_form_dict = open_request_info.from_dict(open_request_info_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


