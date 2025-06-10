/* tslint:disable */
/* eslint-disable */
export enum InstitutionOrdinalColumn {
  Name = 0,
}
export enum LabOrdinalColumn {
  Name = 0,
}
export enum PersonOrdinalColumn {
  Name = 0,
  Email = 1,
}
export enum UserRole {
  AppAdmin = 0,
  ComputationalStaff = 1,
  BiologyStaff = 2,
  Unknown = 3,
}
export class Client {
  free(): void;
  send_new_institution(data: NewInstitution, api_key?: string | null): Promise<Institution>;
  send_new_person(data: NewPerson, api_key?: string | null): Promise<Person>;
  constructor(backend_url: string, token: string);
  send_new_ms_login(data: NewPerson): Promise<CreatedUser>;
}
export class CreatedUser {
  private constructor();
  free(): void;
  person: Person;
  api_key: string;
}
export class Institution {
  private constructor();
  free(): void;
  0: InstitutionSummary;
}
export class InstitutionOrdering {
  private constructor();
/**
** Return copy of self without private attributes.
*/
  toJSON(): Object;
/**
* Return stringified version of self.
*/
  toString(): string;
  free(): void;
  static new(): InstitutionOrderingBuilder;
  column: InstitutionOrdinalColumn;
  descending: boolean;
}
/**
 * Builder for [`InstitutionOrdering`](struct.InstitutionOrdering.html).
 */
export class InstitutionOrderingBuilder {
  private constructor();
  free(): void;
  column(value: InstitutionOrdinalColumn): InstitutionOrderingBuilder;
  descending(value: boolean): InstitutionOrderingBuilder;
  /**
   * Builds a new `InstitutionOrdering`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): InstitutionOrdering;
}
export class InstitutionOrderingError {
  private constructor();
  free(): void;
  error(): string;
}
export class InstitutionQuery {
/**
** Return copy of self without private attributes.
*/
  toJSON(): Object;
/**
* Return stringified version of self.
*/
  toString(): string;
  free(): void;
  constructor();
  ids: string[];
  get name(): string;
  set name(value: string | null | undefined);
  order_by: InstitutionOrdering[];
  pagination: Pagination;
}
export class InstitutionReference {
  private constructor();
  free(): void;
  id: string;
  link: string;
}
export class InstitutionSummary {
  private constructor();
  free(): void;
  reference: InstitutionReference;
  name: string;
}
export class Lab {
  private constructor();
  free(): void;
  summary: LabSummary;
  pi: PersonSummary;
}
export class LabOrdering {
  private constructor();
/**
** Return copy of self without private attributes.
*/
  toJSON(): Object;
/**
* Return stringified version of self.
*/
  toString(): string;
  free(): void;
  static new(): LabOrderingBuilder;
  column: LabOrdinalColumn;
  descending: boolean;
}
/**
 * Builder for [`LabOrdering`](struct.LabOrdering.html).
 */
export class LabOrderingBuilder {
  private constructor();
  free(): void;
  column(value: LabOrdinalColumn): LabOrderingBuilder;
  descending(value: boolean): LabOrderingBuilder;
  /**
   * Builds a new `LabOrdering`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): LabOrdering;
}
export class LabOrderingError {
  private constructor();
  free(): void;
  error(): string;
}
export class LabQuery {
/**
** Return copy of self without private attributes.
*/
  toJSON(): Object;
/**
* Return stringified version of self.
*/
  toString(): string;
  free(): void;
  constructor();
  ids: string[];
  get name(): string;
  set name(value: string | null | undefined);
  order_by: LabOrdering[];
  pagination: Pagination;
}
export class LabReference {
  private constructor();
  free(): void;
}
export class LabSummary {
  private constructor();
  free(): void;
  reference: LabReference;
  name: string;
  delivery_dir: string;
}
export class NewInstitution {
  private constructor();
/**
** Return copy of self without private attributes.
*/
  toJSON(): Object;
/**
* Return stringified version of self.
*/
  toString(): string;
  free(): void;
  static new(): NewInstitutionBuilder;
  id: string;
  name: string;
}
/**
 * Builder for [`NewInstitution`](struct.NewInstitution.html).
 */
export class NewInstitutionBuilder {
  private constructor();
  free(): void;
  id(value: string): NewInstitutionBuilder;
  name(value: string): NewInstitutionBuilder;
  /**
   * Builds a new `NewInstitution`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewInstitution;
}
export class NewInstitutionError {
  private constructor();
  free(): void;
  error(): string;
}
export class NewLab {
  private constructor();
/**
** Return copy of self without private attributes.
*/
  toJSON(): Object;
/**
* Return stringified version of self.
*/
  toString(): string;
  free(): void;
  static new(): NewLabBuilder;
  name: string;
  pi_id: string;
  delivery_dir: string;
  member_ids: string[];
}
/**
 * Builder for [`NewLab`](struct.NewLab.html).
 */
export class NewLabBuilder {
  private constructor();
  free(): void;
  name(value: string): NewLabBuilder;
  pi_id(value: string): NewLabBuilder;
  delivery_dir(value: string): NewLabBuilder;
  member_ids(value: string[]): NewLabBuilder;
  /**
   * Builds a new `NewLab`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewLab;
}
export class NewLabError {
  private constructor();
  free(): void;
  error(): string;
}
export class NewPerson {
  private constructor();
/**
** Return copy of self without private attributes.
*/
  toJSON(): Object;
/**
* Return stringified version of self.
*/
  toString(): string;
  free(): void;
  static new(): NewPersonBuilder;
  name: string;
  email: string;
  get orcid(): string;
  set orcid(value: string | null | undefined);
  institution_id: string;
  get ms_user_id(): string;
  set ms_user_id(value: string | null | undefined);
  roles: any[];
}
/**
 * Builder for [`NewPerson`](struct.NewPerson.html).
 */
export class NewPersonBuilder {
  private constructor();
  free(): void;
  name(value: string): NewPersonBuilder;
  email(value: string): NewPersonBuilder;
  orcid(value?: string | null): NewPersonBuilder;
  institution_id(value: string): NewPersonBuilder;
  ms_user_id(value?: string | null): NewPersonBuilder;
  roles(value: any[]): NewPersonBuilder;
  /**
   * Builds a new `NewPerson`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewPerson;
}
export class NewPersonError {
  private constructor();
  free(): void;
  error(): string;
}
export class Pagination {
/**
** Return copy of self without private attributes.
*/
  toJSON(): Object;
/**
* Return stringified version of self.
*/
  toString(): string;
  free(): void;
  constructor(limit: bigint, offset: bigint);
  limit: bigint;
  offset: bigint;
}
export class Person {
  private constructor();
  free(): void;
  summary: PersonSummary;
  institution: Institution;
}
export class PersonOrdering {
  private constructor();
/**
** Return copy of self without private attributes.
*/
  toJSON(): Object;
/**
* Return stringified version of self.
*/
  toString(): string;
  free(): void;
  static new(): PersonOrderingBuilder;
  column: PersonOrdinalColumn;
  descending: boolean;
}
/**
 * Builder for [`PersonOrdering`](struct.PersonOrdering.html).
 */
export class PersonOrderingBuilder {
  private constructor();
  free(): void;
  column(value: PersonOrdinalColumn): PersonOrderingBuilder;
  descending(value: boolean): PersonOrderingBuilder;
  /**
   * Builds a new `PersonOrdering`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): PersonOrdering;
}
export class PersonOrderingError {
  private constructor();
  free(): void;
  error(): string;
}
export class PersonQuery {
/**
** Return copy of self without private attributes.
*/
  toJSON(): Object;
/**
* Return stringified version of self.
*/
  toString(): string;
  free(): void;
  constructor();
  ids: string[];
  get name(): string;
  set name(value: string | null | undefined);
  get email(): string;
  set email(value: string | null | undefined);
  order_by: PersonOrdering[];
  pagination: Pagination;
}
export class PersonReference {
  private constructor();
  free(): void;
  id: string;
  link: string;
}
export class PersonSummary {
  private constructor();
  free(): void;
  reference: PersonReference;
  name: string;
  get email(): string;
  set email(value: string | null | undefined);
  get orcid(): string;
  set orcid(value: string | null | undefined);
}
