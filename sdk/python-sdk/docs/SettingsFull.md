# SettingsFull

The live settings of kraken

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**mfa_required** | **bool** | Require mfa for local users | 
**oidc_initial_permission_level** | [**UserPermission**](UserPermission.md) |  | 
**dehashed_email** | **str** | The email for the dehashed account | [optional] 
**dehashed_api_key** | **str** | The api key for the dehashed account | [optional] 
**created_at** | **datetime** | The point in time the settings were created | 

## Example

```python
from kraken_sdk.models.settings_full import SettingsFull

# TODO update the JSON string below
json = "{}"
# create an instance of SettingsFull from a JSON string
settings_full_instance = SettingsFull.from_json(json)
# print the JSON string representation of the object
print SettingsFull.to_json()

# convert the object into a dict
settings_full_dict = settings_full_instance.to_dict()
# create an instance of SettingsFull from a dict
settings_full_form_dict = settings_full.from_dict(settings_full_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


