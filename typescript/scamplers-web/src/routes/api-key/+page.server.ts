export function load({ cookies }) {
  const user_id = cookies.get("USER_ID");

  return { user_id };
}
