# kraken_sdk.WordlistApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_all_wordlists**](WordlistApi.md#get_all_wordlists) | **GET** /api/v1/wordlists | Get a list of all wordlist for the user to select from when starting an bruteforce subdomains attack


# **get_all_wordlists**
> GetAllWordlistsResponse get_all_wordlists()

Get a list of all wordlist for the user to select from when starting an bruteforce subdomains attack

Get a list of all wordlist for the user to select from when starting an bruteforce subdomains attack

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_all_wordlists_response import GetAllWordlistsResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.WordlistApi(api_client)

    try:
        # Get a list of all wordlist for the user to select from when starting an bruteforce subdomains attack
        api_response = await api_instance.get_all_wordlists()
        print("The response of WordlistApi->get_all_wordlists:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WordlistApi->get_all_wordlists: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

[**GetAllWordlistsResponse**](GetAllWordlistsResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Matched leeches |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

