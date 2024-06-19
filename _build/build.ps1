docker run -it --name wiclive -v D:\0_WORKBENCH\WICLIVE:C:\WICLIVE wiclive -Command "dir C:/" #-Command "bun i; bun run goes.ts build testing"

# docker run --mount source=wiclive,target=C:\wiclive -w C:\\WICLIVE wiclive 