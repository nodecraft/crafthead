// RequestedKind determines the kind of request the user is making.
export enum RequestedKind {
    Skin,
    Avatar
}

// IdentityKind determines if the request is for a UUID or a username.
export enum IdentityKind {
    Uuid,
    Username
}

export interface MineheadRequest {
    requested: RequestedKind;
    identity: string;
    identityType: IdentityKind;
    size: number;
}

function stringKindToRequestedKind(kind: string): RequestedKind | null {
    switch (kind) {
        case "skin":
            return RequestedKind.Skin;
        case "avatar":
            return RequestedKind.Avatar;
        default:
            return null;
    }
}

export function interpretRequest(request: Request): MineheadRequest | null {
    const url = new URL(request.url)
    if (url.href.endsWith(".png")) {
        url.href = url.href.substring(0, url.href.length - 4)
    }

    let [requestedKindString, identity, sizeString] = url.pathname.split('/').slice(1)
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

    return { requested, identityType, identity, size }
}