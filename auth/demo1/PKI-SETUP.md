# Private Key Infrastruture setup

[link](https://www.ibm.com/docs/en/license-metric-tool?topic=certificate-step-1-creating-private-keys-certificates)

## Private Key instructions

### Generate Private key (PKCS#1 format)

```
openssl genrsa -des3 -out demo-pkey.key 2048
```
demo-pkey.key password: NotaLongPassword

### Create a Certificate Signing Request (CSR)

```
openssl req -new -key demo-pkey.key -out demo-CSR.csr
```
Challenge password: NotaChallenge

## Certificate Authority instrutions

### Generate Private key & CSR for Certificate Authority (CA)

```
openssl req -new -newkey rsa:2048 -nodes -out demo-CA-CSR.csr -keyout demo-CA-pkey.key -sha256
```
Challenge password: CAChallengePassword

### Generate Certificate for CA & sign CA-CSR

```
openssl x509 -signkey demo-CA-pkey.key -days 356 -req -in demo-CA-CSR.csr -out demo-CA-certificate.arm -sha256
```

### Sign CA-CSR using above Certificate

```
openssl x509 -req -days 90 -in demo-pkey-CSR.csr -CA demo-CA-certificate.arm -CAkey demo-CA-pkey.key -out demo-pkey-certificate.arm -set_serial 01 -sha256
```

## Epilogue

This has created two certificates, `demo-pkey-certificate.arm` for the private key and `demo-CA-certificate.arm` for the Certificate Authority.
