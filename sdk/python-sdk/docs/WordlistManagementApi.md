# kraken_sdk.WordlistManagementApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_wordlist_admin**](WordlistManagementApi.md#create_wordlist_admin) | **POST** /api/v1/admin/wordlists | Create a new wordlist
[**delete_wordlist_admin**](WordlistManagementApi.md#delete_wordlist_admin) | **DELETE** /api/v1/admin/wordlists/{uuid} | Delete an existing wordlist
[**get_all_wordlists_admin**](WordlistManagementApi.md#get_all_wordlists_admin) | **GET** /api/v1/admin/wordlists | Get a list of all wordlists including their paths
[**update_wordlist_admin**](WordlistManagementApi.md#update_wordlist_admin) | **PUT** /api/v1/admin/wordlists/{uuid} | Update an existing wordlist


# **create_wordlist_admin**
> UuidResponse create_wordlist_admin(create_wordlist_request)

Create a new wordlist

Create a new wordlist

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.create_wordlist_request import CreateWordlistRequest
from kraken_sdk.models.uuid_response import UuidResponse
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
    api_instance = kraken_sdk.WordlistManagementApi(api_client)
    create_wordlist_request = kraken_sdk.CreateWordlistRequest() # CreateWordlistRequest | 

    try:
        # Create a new wordlist
        api_response = await api_instance.create_wordlist_admin(create_wordlist_request)
        print("The response of WordlistManagementApi->create_wordlist_admin:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WordlistManagementApi->create_wordlist_admin: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_wordlist_request** | [**CreateWordlistRequest**](CreateWordlistRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Wordlist got created successfully |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_wordlist_admin**
> delete_wordlist_admin(uuid)

Delete an existing wordlist

Delete an existing wordlist

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
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
    api_instance = kraken_sdk.WordlistManagementApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Delete an existing wordlist
        await api_instance.delete_wordlist_admin(uuid)
    except Exception as e:
        print("Exception when calling WordlistManagementApi->delete_wordlist_admin: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Wordlist got deleted |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_all_wordlists_admin**
> GetAllWordlistsAdminResponse get_all_wordlists_admin()

Get a list of all wordlists including their paths

Get a list of all wordlists including their paths

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_all_wordlists_admin_response import GetAllWordlistsAdminResponse
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
    api_instance = kraken_sdk.WordlistManagementApi(api_client)

    try:
        # Get a list of all wordlists including their paths
        api_response = await api_instance.get_all_wordlists_admin()
        print("The response of WordlistManagementApi->get_all_wordlists_admin:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WordlistManagementApi->get_all_wordlists_admin: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

[**GetAllWordlistsAdminResponse**](GetAllWordlistsAdminResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | List of all wordlists |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_wordlist_admin**
> update_wordlist_admin(uuid, update_wordlist_request)

Update an existing wordlist

Update an existing wordlist

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.update_wordlist_request import UpdateWordlistRequest
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
    api_instance = kraken_sdk.WordlistManagementApi(api_client)
    uuid = 'uuid_example' # str | 
    update_wordlist_request = kraken_sdk.UpdateWordlistRequest() # UpdateWordlistRequest | 

    try:
        # Update an existing wordlist
        await api_instance.update_wordlist_admin(uuid, update_wordlist_request)
    except Exception as e:
        print("Exception when calling WordlistManagementApi->update_wordlist_admin: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **update_wordlist_request** | [**UpdateWordlistRequest**](UpdateWordlistRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Wordlist got updated |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

