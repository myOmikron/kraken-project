# FullWordlist

A wordlist including its `path` field only meant for admins

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key of the wordlist | 
**name** | **str** | The wordlist&#39;s name to be displayed select buttons | 
**description** | **str** | A description explaining the wordlist&#39;s intended use case | 
**path** | **str** | The file path the wordlist is deployed under on each leech | 

## Example

```python
from kraken_sdk.models.full_wordlist import FullWordlist

# TODO update the JSON string below
json = "{}"
# create an instance of FullWordlist from a JSON string
full_wordlist_instance = FullWordlist.from_json(json)
# print the JSON string representation of the object
print FullWordlist.to_json()

# convert the object into a dict
full_wordlist_dict = full_wordlist_instance.to_dict()
# create an instance of FullWordlist from a dict
full_wordlist_form_dict = full_wordlist.from_dict(full_wordlist_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


