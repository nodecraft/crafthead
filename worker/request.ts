// RequestedKind determines the kind of request the user is making.
export enum RequestedKind {
    Skin,
    Avatar,
    Helm,
    Cube,
    Body,
    Bust,
    Cape,
    Profile
}

// IdentityKind determines if the request is for a UUID or a username.
export enum IdentityKind {
    Uuid,
    Username
}

export enum TextureKind {
    SKIN,
    CAPE
}

export interface CraftheadRequest {
    requested: RequestedKind;
    identity: string;
    identityType: IdentityKind;
    size: number;
    armored: boolean;
    model: string | null;
}

function stringKindToRequestedKind(kind: string): RequestedKind | null {
    switch (kind) {
        case "skin":
            return RequestedKind.Skin;
        case "avatar":
            return RequestedKind.Avatar;
        case "cube":
            return RequestedKind.Cube;
        case "helm":
            return RequestedKind.Helm;
        case "body":
            return RequestedKind.Body;
        case "bust":
            return RequestedKind.Bust;
        case "cape":
            return RequestedKind.Cape;
        case "profile":
            return RequestedKind.Profile;
        default:
            return null;
    }
}

export function interpretRequest(request: Request): CraftheadRequest | null {
    const url = new URL(request.url)
    if (url.href.endsWith(".png")) {
        url.href = url.href.substring(0, url.href.length - 4)
    }

    let model = url.searchParams.get("model")
    if (model && !["slim", "default"].includes(model)) model = null

    let armored = false
    let sliceAmt = 1

    if (url.pathname.includes("/armor/cube/") || url.pathname.includes("/armor/body/") || url.pathname.includes("/armor/bust/")) {
        armored = true
        sliceAmt = 2
    }

    let [requestedKindString, identity, sizeString] = url.pathname.split('/').slice(sliceAmt)

    let size = parseInt(sizeString, 10);
    if (!size) {
        size = 180 // default, same as Minotar
    } else if (size < 8) {
        size = 8 // minimum size
    } else if (size > 300) {
        // In order to limit abuse, don't scale above 300px.
        size = 300
    }

    const requested = stringKindToRequestedKind(requestedKindString);
    if (requested == null) {
        return null
    }

    let identityType: IdentityKind
    if (identity.length <= 16) {
        identityType = IdentityKind.Username
    } else if (identity.length === 32) {
        identityType = IdentityKind.Uuid
    } else if (identity.length === 36) {
        identity = identity.replace(/-/g, '')
        identityType = IdentityKind.Uuid
    } else {
        return null
    }

    return { requested, identityType, identity, size, armored, model }
}