FROM scratch

COPY ./musl_release /codenamer
COPY static /static

CMD ["/codenamer"]