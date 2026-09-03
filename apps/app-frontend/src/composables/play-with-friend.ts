export type PlayWithFriendMember = {
	id: number
	minecraft_nick: string
	join_address?: string | null
	instance_name?: string | null
	pack_project_id?: string | null
	pack_version_id?: string | null
	presence?: string
}

export function canPlayWithFriend(member: PlayWithFriendMember) {
	return member.presence === 'ingame' && !!member.join_address?.trim()
}

export function canViewFriendPack(member: PlayWithFriendMember) {
	if (member.pack_project_id?.trim()) return true
	return member.presence === 'ingame' && !!member.instance_name?.trim()
}
