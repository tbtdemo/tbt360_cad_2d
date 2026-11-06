// A pure host-side policy so CI can test tenant/role and loopback input
// without an AuthServer, a running CAD process or any real credentials.
public static class HostGrantPolicy
{
    public static bool MayRead(string? subject,string? tenant,string? role,string? boundTenant) =>
        !string.IsNullOrWhiteSpace(subject)
        && !string.IsNullOrWhiteSpace(tenant)
        && !string.IsNullOrWhiteSpace(boundTenant)
        && string.Equals(tenant,boundTenant,StringComparison.Ordinal)
        && (role is "viewer" or "editor" or "approver");

    public static bool MayPreview(string? subject,string? tenant,string? role,string? boundTenant) =>
        MayRead(subject,tenant,role,boundTenant) && (role is "editor" or "approver");

    public static bool ValidPreviewText(string? text) =>
        !string.IsNullOrWhiteSpace(text) && text.Length <= 4000 && !text.Any(char.IsControl);

    public static bool IsLoopbackAddress(string address) =>
        address.StartsWith("127.0.0.1:",StringComparison.Ordinal)
        && int.TryParse(address.AsSpan("127.0.0.1:".Length),out var port)
        && port is >= 1 and <= 65535;

    public static int SelfTest()
    {
        var checks=new (bool Success,string Name)[] {
            (MayRead("u1","t1","viewer","t1"),"same-tenant viewer"),
            (MayRead("u1","t1","editor","t1"),"same-tenant editor"),
            (!MayRead("u1","t1","approver","t2"),"cross-tenant forbidden"),
            (!MayRead("u1","t1","owner","t1"),"unknown role forbidden"),
            (!MayRead("","t1","viewer","t1"),"anonymous forbidden"),
            (!MayRead("u1","t1","viewer",null),"missing server-tenant forbidden"),
            (MayPreview("u1","t1","editor","t1"),"same-tenant editor may preview"),
            (MayPreview("u1","t1","approver","t1"),"same-tenant approver may preview"),
            (!MayPreview("u1","t1","viewer","t1"),"viewer cannot preview"),
            (!MayPreview("u1","t1","editor","other-tenant"),"cross-tenant preview forbidden"),
            (ValidPreviewText("vẽ hai phòng liền kề rộng 3000 và 4000 dài 4000 tại 0,0"),"bounded preview allowed"),
            (!ValidPreviewText(""),"empty preview denied"),
            (!ValidPreviewText(new string('x',4001)),"oversized preview denied"),
            (!ValidPreviewText("line\nscript"),"control characters denied"),
            (IsLoopbackAddress("127.0.0.1:7979"),"local loopback allowed"),
            (!IsLoopbackAddress("0.0.0.0:7979"),"public bind forbidden"),
            (!IsLoopbackAddress("localhost:7979"),"ambiguous host forbidden"),
            (!IsLoopbackAddress("127.0.0.1:0"),"invalid port forbidden"),
        };
        foreach(var (success,name) in checks) {
            if (!success) {Console.Error.WriteLine("FAIL host policy: "+name);return 1;}
            Console.WriteLine("PASS host policy: "+name);
        }
        return 0;
    }
}
