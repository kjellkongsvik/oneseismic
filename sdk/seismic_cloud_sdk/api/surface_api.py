# coding: utf-8

"""
    Seismic Cloud Api

    The Seismic Cloud Api  # noqa: E501

    OpenAPI spec version: 1.0.0
    
    Generated by: https://github.com/swagger-api/swagger-codegen.git
"""


from __future__ import absolute_import

import re  # noqa: F401

# python 2 and python 3 compatibility library
import six

from seismic_cloud_sdk.api_client import ApiClient


class SurfaceApi(object):
    """NOTE: This class is auto generated by the swagger code generator program.

    Do not edit the class manually.
    Ref: https://github.com/swagger-api/swagger-codegen
    """

    def __init__(self, api_client=None):
        if api_client is None:
            api_client = ApiClient()
        self.api_client = api_client

    def download_surface(self, surface_id, **kwargs):  # noqa: E501
        """download_surface  # noqa: E501

        get surface file  # noqa: E501
        This method makes a synchronous HTTP request by default. To make an
        asynchronous HTTP request, please pass async_req=True
        >>> thread = api.download_surface(surface_id, async_req=True)
        >>> result = thread.get()

        :param async_req bool
        :param str surface_id: File ID (required)
        :return: str
                 If the method is called asynchronously,
                 returns the request thread.
        """
        kwargs['_return_http_data_only'] = True
        if kwargs.get('async_req'):
            return self.download_surface_with_http_info(surface_id, **kwargs)  # noqa: E501
        else:
            (data) = self.download_surface_with_http_info(surface_id, **kwargs)  # noqa: E501
            return data

    def download_surface_with_http_info(self, surface_id, **kwargs):  # noqa: E501
        """download_surface  # noqa: E501

        get surface file  # noqa: E501
        This method makes a synchronous HTTP request by default. To make an
        asynchronous HTTP request, please pass async_req=True
        >>> thread = api.download_surface_with_http_info(surface_id, async_req=True)
        >>> result = thread.get()

        :param async_req bool
        :param str surface_id: File ID (required)
        :return: str
                 If the method is called asynchronously,
                 returns the request thread.
        """

        all_params = ['surface_id']  # noqa: E501
        all_params.append('async_req')
        all_params.append('_return_http_data_only')
        all_params.append('_preload_content')
        all_params.append('_request_timeout')

        params = locals()
        for key, val in six.iteritems(params['kwargs']):
            if key not in all_params:
                raise TypeError(
                    "Got an unexpected keyword argument '%s'"
                    " to method download_surface" % key
                )
            params[key] = val
        del params['kwargs']
        # verify the required parameter 'surface_id' is set
        if ('surface_id' not in params or
                params['surface_id'] is None):
            raise ValueError("Missing the required parameter `surface_id` when calling `download_surface`")  # noqa: E501

        collection_formats = {}

        path_params = {}
        if 'surface_id' in params:
            path_params['surfaceID'] = params['surface_id']  # noqa: E501

        query_params = []

        header_params = {}

        form_params = []
        local_var_files = {}

        body_params = None
        # HTTP header `Accept`
        header_params['Accept'] = self.api_client.select_header_accept(
            ['application/octet-stream'])  # noqa: E501

        # Authentication setting
        auth_settings = ['ApiKeyAuth']  # noqa: E501

        return self.api_client.call_api(
            '/surface/{surfaceID}', 'GET',
            path_params,
            query_params,
            header_params,
            body=body_params,
            post_params=form_params,
            files=local_var_files,
            response_type='str',  # noqa: E501
            auth_settings=auth_settings,
            async_req=params.get('async_req'),
            _return_http_data_only=params.get('_return_http_data_only'),
            _preload_content=params.get('_preload_content', True),
            _request_timeout=params.get('_request_timeout'),
            collection_formats=collection_formats)

    def list_surfaces(self, **kwargs):  # noqa: E501
        """list_surfaces  # noqa: E501

        get list of available surfaces  # noqa: E501
        This method makes a synchronous HTTP request by default. To make an
        asynchronous HTTP request, please pass async_req=True
        >>> thread = api.list_surfaces(async_req=True)
        >>> result = thread.get()

        :param async_req bool
        :return: list[StoreSurfaceMeta]
                 If the method is called asynchronously,
                 returns the request thread.
        """
        kwargs['_return_http_data_only'] = True
        if kwargs.get('async_req'):
            return self.list_surfaces_with_http_info(**kwargs)  # noqa: E501
        else:
            (data) = self.list_surfaces_with_http_info(**kwargs)  # noqa: E501
            return data

    def list_surfaces_with_http_info(self, **kwargs):  # noqa: E501
        """list_surfaces  # noqa: E501

        get list of available surfaces  # noqa: E501
        This method makes a synchronous HTTP request by default. To make an
        asynchronous HTTP request, please pass async_req=True
        >>> thread = api.list_surfaces_with_http_info(async_req=True)
        >>> result = thread.get()

        :param async_req bool
        :return: list[StoreSurfaceMeta]
                 If the method is called asynchronously,
                 returns the request thread.
        """

        all_params = []  # noqa: E501
        all_params.append('async_req')
        all_params.append('_return_http_data_only')
        all_params.append('_preload_content')
        all_params.append('_request_timeout')

        params = locals()
        for key, val in six.iteritems(params['kwargs']):
            if key not in all_params:
                raise TypeError(
                    "Got an unexpected keyword argument '%s'"
                    " to method list_surfaces" % key
                )
            params[key] = val
        del params['kwargs']

        collection_formats = {}

        path_params = {}

        query_params = []

        header_params = {}

        form_params = []
        local_var_files = {}

        body_params = None
        # HTTP header `Accept`
        header_params['Accept'] = self.api_client.select_header_accept(
            ['application/json'])  # noqa: E501

        # Authentication setting
        auth_settings = ['ApiKeyAuth']  # noqa: E501

        return self.api_client.call_api(
            '/surface/', 'GET',
            path_params,
            query_params,
            header_params,
            body=body_params,
            post_params=form_params,
            files=local_var_files,
            response_type='list[StoreSurfaceMeta]',  # noqa: E501
            auth_settings=auth_settings,
            async_req=params.get('async_req'),
            _return_http_data_only=params.get('_return_http_data_only'),
            _preload_content=params.get('_preload_content', True),
            _request_timeout=params.get('_request_timeout'),
            collection_formats=collection_formats)