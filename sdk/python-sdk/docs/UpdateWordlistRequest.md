# UpdateWordlistRequest

Arguments for updating an existing wordlist

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key of the wordlist to update | 
**name** | **str** | The wordlist&#39;s name to be displayed select buttons | [optional] 
**description** | **str** | A description explaining the wordlist&#39;s intended use case | [optional] 
**path** | **str** | The file path the wordlist is deployed under on each leech | [optional] 

## Example

```python
from kraken_sdk.models.update_wordlist_request import UpdateWordlistRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateWordlistRequest from a JSON string
update_wordlist_request_instance = UpdateWordlistRequest.from_json(json)
# print the JSON string representation of the object
print UpdateWordlistRequest.to_json()

# convert the object into a dict
update_wordlist_request_dict = update_wordlist_request_instance.to_dict()
# create an instance of UpdateWordlistRequest from a dict
update_wordlist_request_form_dict = update_wordlist_request.from_dict(update_wordlist_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


