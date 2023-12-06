# GetAllWordlistsAdminResponse

Response containing all wordlists including their `path` fields

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**wordlists** | [**List[FullWordlist]**](FullWordlist.md) | List of all wordlists including their &#x60;path&#x60; fields | 

## Example

```python
from kraken_sdk.models.get_all_wordlists_admin_response import GetAllWordlistsAdminResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetAllWordlistsAdminResponse from a JSON string
get_all_wordlists_admin_response_instance = GetAllWordlistsAdminResponse.from_json(json)
# print the JSON string representation of the object
print GetAllWordlistsAdminResponse.to_json()

# convert the object into a dict
get_all_wordlists_admin_response_dict = get_all_wordlists_admin_response_instance.to_dict()
# create an instance of GetAllWordlistsAdminResponse from a dict
get_all_wordlists_admin_response_form_dict = get_all_wordlists_admin_response.from_dict(get_all_wordlists_admin_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


