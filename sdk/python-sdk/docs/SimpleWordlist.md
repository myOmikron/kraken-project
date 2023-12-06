# SimpleWordlist

A wordlist without its `path` field

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key of the wordlist | 
**name** | **str** | The wordlist&#39;s name to be displayed select buttons | 
**description** | **str** | A description explaining the wordlist&#39;s intended use case | 

## Example

```python
from kraken_sdk.models.simple_wordlist import SimpleWordlist

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleWordlist from a JSON string
simple_wordlist_instance = SimpleWordlist.from_json(json)
# print the JSON string representation of the object
print SimpleWordlist.to_json()

# convert the object into a dict
simple_wordlist_dict = simple_wordlist_instance.to_dict()
# create an instance of SimpleWordlist from a dict
simple_wordlist_form_dict = simple_wordlist.from_dict(simple_wordlist_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


