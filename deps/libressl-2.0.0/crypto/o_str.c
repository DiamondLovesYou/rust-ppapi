/* $OpenBSD: o_str.c,v 1.8 2014/06/12 15:49:27 deraadt Exp $ */
/*
 * Written by Theo de Raadt.  Public domain.
 */

#include <string.h>

int OPENSSL_strcasecmp(const char *str1, const char *str2);
int OPENSSL_strncasecmp(const char *str1, const char *str2, size_t n);

int
OPENSSL_strncasecmp(const char *str1, const char *str2, size_t n)
{
#if !defined(__pnacl__)
	return strncasecmp(str1, str2, n);
#else
        size_t i = 0;
        for(; str1[i] == str2[i] &&
              str1[i] != '\0' && str2[i] != '\0' &&
              i < n; ++i) { }
        return (int)(str1[i] - str2[i]);
#endif
}

int
OPENSSL_strcasecmp(const char *str1, const char *str2)
{
#if !defined(__pnacl__)
	return strcasecmp(str1, str2);
#else
        size_t i = 0;
        for(; str1[i] == str2[i] &&
              str1[i] != '\0' && str2[i] != '\0'; ++i) { }
        return (int)(str1[i] - str2[i]);
#endif
}
