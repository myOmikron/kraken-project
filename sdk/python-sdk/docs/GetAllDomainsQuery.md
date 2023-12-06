# GetAllDomainsQuery

Query parameters for filtering the domains to get

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**limit** | **int** | Number of items to retrieve | 
**offset** | **int** | Position in the whole list to start retrieving from | 
**host** | **str** | Only get domains pointing to a specific host  This includes domains which point to another domain which points to this host. | [optional] 
**global_filter** | **str** | An optional general filter to apply | [optional] 
**domain_filter** | **str** | An optional domain specific filter to apply | [optional] 

## Example

```python
from kraken_sdk.models.get_all_domains_query import GetAllDomainsQuery

# TODO update the JSON string below
json = "{}"
# create an instance of GetAllDomainsQuery from a JSON string
get_all_domains_query_instance = GetAllDomainsQuery.from_json(json)
# print the JSON string representation of the object
print GetAllDomainsQuery.to_json()

# convert the object into a dict
get_all_domains_query_dict = get_all_domains_query_instance.to_dict()
# create an instance of GetAllDomainsQuery from a dict
get_all_domains_query_form_dict = get_all_domains_query.from_dict(get_all_domains_query_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


