# ProjectSureEff
 This is a student project
 from
 Quentin Soutelo
 Emilien Valain


Ce projet est un jeu du pendu fonctionnant en multijoueur avec un serveur et des clients.

Pour jouer au jeu l'utilisateur doit lancer le serveur via la commande ```cargo run``` dans le fichier serveur, puis démarrer autant de client qu'il le souhaite, toujours via la commande ```cargo run```, mais cette fois dans le fichier client.

Le but des clients est de trouver le mot caché, tous les clients ont le même mot, même lettre utilisée et le même nombre d'essai qui est partagé entre tous les clients.
Ce qui signifie que si un client donne la lettre "e" par exemple les autres n'auront pas à la donner et seront informés que la lettre a déjà été trouvée.

Dans l'état de ce projet la communication n'est pas instantanée entre les clients, ils recevront les mises à jour des autres clients que quand il donne sa propre lettre.
Le jeu n'a pas de rejouabilité facile, pour rejouer il faut impérativement relancer le serveur et les clients.
Enfin le jeu ne fonctionne que sur une seule machine n'ayant pas réussi à implémenter une communication en ligne.
