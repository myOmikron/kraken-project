# UpdateSettingsRequest

The request to update the settings

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**mfa_required** | **bool** | Require mfa for local users | 
**oidc_initial_permission_level** | [**UserPermission**](UserPermission.md) |  | 
**dehashed_email** | **str** | The email for the dehashed account | [optional] 
**dehashed_api_key** | **str** | The api key for the dehashed account | [optional] 

## Example

```python
from kraken_sdk.models.update_settings_request import UpdateSettingsRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateSettingsRequest from a JSON string
update_settings_request_instance = UpdateSettingsRequest.from_json(json)
# print the JSON string representation of the object
print UpdateSettingsRequest.to_json()

# convert the object into a dict
update_settings_request_dict = update_settings_request_instance.to_dict()
# create an instance of UpdateSettingsRequest from a dict
update_settings_request_form_dict = update_settings_request.from_dict(update_settings_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


